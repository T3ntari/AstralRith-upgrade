use crate::event::emit::{emit_loading, init_loading};
use crate::{LoadingBarType, Result};
use futures::StreamExt;
use reqwest;
use std::path::PathBuf;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Downloads a launcher update with real progress events and returns the path
/// where the artifact was saved.
///
/// Progress is streamed to the frontend through the existing `loading` event
/// system (LoadingBarType::LauncherUpdate), so the update dialog can show a
/// true percentage + ETA instead of a fake animated bar.
async fn download_file(
    download_url: &str,
    local_filename: &str,
    version: &str,
    current_version: &str,
) -> Result<PathBuf> {
    let download_dir = dirs::download_dir()
        .ok_or_else(|| crate::Error::from(crate::ErrorKind::OtherError(
            "Failed to determine download directory".to_string(),
        )))?;
    let full_path = download_dir.join(local_filename);

    let client = reqwest::Client::new();
    let response = client.get(download_url).send().await.map_err(|e| {
        crate::Error::from(crate::ErrorKind::OtherError(format!(
            "[download_file] • Request failed: {e}"
        )))
    })?;

    let total_size = response
        .content_length()
        .unwrap_or(0) as f64;

    // Loading bar drives the update dialog's real progress.
    let loading_bar = init_loading(
        LoadingBarType::LauncherUpdate {
            version: version.to_string(),
            current_version: current_version.to_string(),
        },
        100.0,
        "Downloading update...",
    )
    .await?;

    let mut stream = response.bytes_stream();
    let mut dest_file = AsyncFile::create(&full_path).await.map_err(|e| {
        crate::Error::from(crate::ErrorKind::OtherError(format!(
            "[download_file] • Cannot create file: {e}"
        )))
    })?;

    let mut downloaded: f64 = 0.0;
    let mut last_emit: f64 = -1.0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            crate::Error::from(crate::ErrorKind::OtherError(format!(
                "[download_file] • Stream error: {e}"
            )))
        })?;
        dest_file.write_all(&chunk).await.map_err(|e| {
            crate::Error::from(crate::ErrorKind::OtherError(format!(
                "[download_file] • Write error: {e}"
            )))
        })?;
        downloaded += chunk.len() as f64;

        // Emit at most every ~1% (or every chunk when no content-length).
        let frac = if total_size > 0.0 {
            (downloaded / total_size * 100.0).min(99.0)
        } else {
            0.0
        };
        if frac - last_emit >= 1.0 || (total_size == 0.0 && downloaded % (1024.0 * 512.0) < 4096.0) {
            last_emit = frac;
            let msg = if total_size > 0.0 {
                format!("Downloading update... {:.0}%", frac)
            } else {
                format!("Downloading update... {:.1} MiB", downloaded / (1024.0 * 1024.0))
            };
            // Ignore emit errors; the bar is best-effort.
            let _ = emit_loading(&loading_bar, frac, Some(&msg));
        }
    }
    dest_file.flush().await.map_err(|e| {
        crate::Error::from(crate::ErrorKind::OtherError(format!(
            "[download_file] • Flush error: {e}"
        )))
    })?;

    // 100% - drop the loading bar (its Drop emits "Completed").
    drop(loading_bar);

    println!("[download_file] • File downloaded to: {:?}", full_path);
    Ok(full_path)
}

/// Launches the installer for the current OS after a download completes.
/// Returns true if the install/launch was handed off successfully.
async fn launch_installer(full_path: &PathBuf, os_type: &str) -> bool {
    let path_str = full_path.to_str().unwrap_or_default();
    let lower = os_type.to_lowercase();

    let result = if lower.contains("windows") {
        // Windows: open the downloads folder (the .msi will show the installer).
        Command::new("explorer")
            .arg(full_path.parent().map(|p| p.display().to_string()).unwrap_or_default())
            .status()
            .await
    } else if lower.contains("mac") {
        // macOS: open the .dmg directly.
        Command::new("open").arg(path_str).status().await
    } else {
        // Linux: install the .deb/.rpm in place, or fall back to xdg-open.
        let filename = full_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if filename.ends_with(".deb") {
            // Try pkexec/gksudo dpkg first (graphical), fall back to plain dpkg.
            let mut cmd = Command::new("pkexec");
            cmd.arg("dpkg").arg("-i").arg(path_str);
            let res = cmd.status().await;
            if let Ok(status) = &res {
                if status.success() {
                    return true;
                }
            }
            let res2 = Command::new("dpkg").arg("-i").arg(path_str).status().await;
            if let Ok(status) = &res2 {
                if status.success() {
                    return true;
                }
            }
            // Last resort: open with the default .deb handler.
            Command::new("xdg-open").arg(path_str).status().await
        } else if filename.ends_with(".rpm") {
            Command::new("pkexec")
                .arg("rpm").arg("-Uvh").arg(path_str)
                .status()
                .await
        } else {
            // AppImage / unknown: just open it.
            Command::new("xdg-open").arg(path_str).status().await
        }
    };

    match result {
        Ok(status) => {
            println!("[download_file] • Installer launch exit code: {:?}", status.code());
            status.success()
        }
        Err(e) => {
            eprintln!("[download_file] • Failed to launch installer: {e}");
            // Fall back to xdg-open so the user can at least open the file.
            let _ = Command::new("xdg-open").arg(path_str).status().await;
            false
        }
    }
}

/// Public entrypoint used by the Tauri `get_artifact` command.
pub async fn init_download(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
    auto_update_supported: bool,
) -> Result<PathBuf> {
    println!("[init_download] • Downloading from • {download_url}");
    println!("[init_download] • Saving as • {local_filename}");

    // Version strings are passed via the filename convention when available
    // (the frontend sends `filename` like "AstralRinth.App_0.9.205_amd64.deb").
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let version = current_version.clone();

    let full_path =
        download_file(download_url, local_filename, &version, &current_version).await?;
    println!("[init_download] • Download completed successfully: {full_path:?}");

    if auto_update_supported {
        let launched = launch_installer(&full_path, os_type).await;
        if launched {
            println!("[init_download] • Installer launched.");
        } else {
            println!("[init_download] • Installer could not be launched automatically.");
        }
    } else {
        println!("[init_download] • Auto-update not supported for this build; file saved to disk.");
    }

    Ok(full_path)
}
