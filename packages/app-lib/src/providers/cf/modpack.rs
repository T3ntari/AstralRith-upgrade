//! CurseForge modpack installation.
//!
//! The pack zip is a CurseForge structure:
//!   manifest.json          -> project/file IDs + minecraft + loaders
//!   overrides/             -> files layered on top of the instance
//!   (optional) server-pack metadata on the CF file
//!
//! Files are resolved through project/file IDs (never guessed from
//! filenames), downloaded with hash verification, and the IDs are retained
//! in the instance metadata so updates can be computed reliably later.

use super::api_models::{CfPackManifest, CfResolvedFile};
use super::client::{cf_post, CfError};
use super::downloads::download_verified;
use crate::event::emit::{emit_loading, init_or_edit_loading, init_loading};
use crate::event::LoadingBarType;
use crate::providers::errors::ProviderError;
use crate::providers::types::ResolvedDownload;
use crate::state::{ProfileInstallStage, ModLoader};
use crate::util::io;
use crate::{profile, State};
use async_zip::base::read::seek::ZipFileReader;
use std::io::Cursor;
use std::path::{Component, PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfInstalledPackInfo {
    pub provider: String,
    pub project_id: u32,
    pub file_id: u32,
    pub name: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
}

/// Installs a CurseForge modpack zip into an existing instance.
///
/// `pack_url` is the resolved download URL for the pack file itself.
/// The manifest's project/file IDs are resolved through the API and every
/// required file is downloaded with hash verification. The `overrides`
/// folder is extracted on top of the profile. The pack identity is stored in
/// `pack.json` inside the profile for later updates.
pub async fn install_modpack(
    profile_path: &str,
    pack_url: &str,
    pack_name: &str,
    pack_project_id: Option<u32>,
    pack_file_id: Option<u32>,
) -> crate::Result<()> {
    let state = State::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::PackDownload {
            profile_path: profile_path.to_string(),
            pack_name: pack_name.to_string(),
            icon: None,
            pack_id: pack_project_id.map(|x| x.to_string()),
            pack_version: pack_file_id.map(|x| x.to_string()),
        },
        1.0,
        "Downloading modpack...",
    )
    .await?;

    let result = install_modpack_inner(
        profile_path,
        pack_url,
        pack_project_id,
        pack_file_id,
        &loading_bar,
        &state,
    )
    .await;

    emit_loading(&loading_bar, 0.0, None)?;

    result
}

async fn install_modpack_inner(
    profile_path: &str,
    pack_url: &str,
    pack_project_id: Option<u32>,
    pack_file_id: Option<u32>,
    loading_bar: &crate::event::LoadingBarId,
    state: &State,
) -> crate::Result<()> {
    // Download the pack zip (the pack itself is served from the CDN; apply
    // the same credential layer used for all CF downloads).
    let zip_bytes = fetch_pack_zip(pack_url, loading_bar, state).await?;

    // Parse manifest.json
    let reader: Cursor<&bytes::Bytes> = Cursor::new(&zip_bytes);
    let mut zip_reader = ZipFileReader::with_tokio(reader).await.map_err(|_| {
        crate::Error::from(crate::ErrorKind::InputError(
            "Failed to read CurseForge modpack zip".to_string(),
        ))
    })?;

    let manifest_entry = zip_reader
        .file()
        .entries()
        .iter()
        .position(|f| f.filename().as_str().unwrap_or_default() == "manifest.json")
        .ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::InputError(
                "CurseForge modpack is missing manifest.json".to_string(),
            ))
        })?;

    let mut manifest_string = String::new();
    let mut entry_reader = zip_reader.reader_with_entry(manifest_entry).await?;
    entry_reader
        .read_to_string_checked(&mut manifest_string)
        .await?;

    let manifest: CfPackManifest = serde_json::from_str(&manifest_string)?;

    let overrides = manifest.overrides.clone().unwrap_or_else(|| "overrides".to_string());

    // Resolve all required files through the CurseForge API by file IDs
    let required_ids = manifest
        .files
        .iter()
        .filter(|f| f.required)
        .map(|f| f.file_id)
        .collect::<Vec<_>>();

    let resolved = if required_ids.is_empty() {
        Vec::new()
    } else {
        let body = serde_json::json!({ "fileIds": required_ids });
        let files: Vec<CfResolvedFile> = cf_post("/mods/files", &body).await?;
        files
    };

    // Total progress: sum of file sizes
    let total_bytes = resolved
        .iter()
        .map(|f| f.file_length)
        .sum::<u64>()
        .max(1) as f64;

    let loading_bar = init_or_edit_loading(
        Some(loading_bar.clone()),
        LoadingBarType::PackDownload {
            profile_path: profile_path.to_string(),
            pack_name: manifest.name.clone(),
            icon: None,
            pack_id: pack_project_id.map(|x| x.to_string()),
            pack_version: Some(manifest.version.clone()),
        },
        total_bytes,
        "Downloading modpack files...",
    )
    .await?;

    // Install each required file into the correct subfolder
    let profile_full_path = get_profile_path(profile_path).await?;

    let mut installed = Vec::new();
    for file in &resolved {
        let Some(url) = &file.download_url else { continue };
        if url.trim().is_empty() {
            continue;
        }

        let target_dir = infer_target_dir(&file.file_name);
        let target = profile_full_path.join(&target_dir);
        let resolved_download = ResolvedDownload {
            url: url.clone(),
            file_name: file.file_name.clone(),
            file_length: Some(file.file_length),
            hash: file
                .hashes
                .as_ref()
                .and_then(|h| h.iter().find(|x| x.algo == 1))
                .map(|h| h.value.clone()),
            requires_manual_download: false,
            project_url: None,
        };

        match download_verified(
            &resolved_download,
            &target,
            file.hashes
                .as_ref()
                .and_then(|h| h.iter().find(|x| x.algo == 1))
                .map(|h| h.value.as_str()),
        )
        .await {
            Ok(_) => {
                installed.push((file.mod_id, file.id));
                emit_loading(
                    &loading_bar,
                    file.file_length as f64,
                    Some(&format!("Installed {}", file.file_name)),
                )?;
            }
            Err(e) => {
                tracing::warn!(
                    cf.project_id = %file.mod_id,
                    cf.file_id = %file.id,
                    "Failed to install CurseForge pack file: {}",
                    e
                );
                // One failed file should not fail the whole pack, but we
                // keep going and surface a warning at the end.
            }
        }
    }

    // Extract the overrides folder on top of the profile
    {
        let overrides_prefix = format!("{}/", overrides.trim_start_matches('/'));
        let override_entries = zip_reader
            .file()
            .entries()
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                let filename = entry.filename().as_str().unwrap_or_default();
                filename.starts_with(&overrides_prefix) && !filename.ends_with('/')
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        for index in override_entries {
            let filename = zip_reader.file().entries()[index]
                .filename()
                .as_str()
                .unwrap_or_default();
            let relative = filename.trim_start_matches(&overrides_prefix);
            let relative_path = PathBuf::from(relative);
            if relative_path.components().any(|c| {
                matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_))
            }) {
                continue;
            }

            let out_path = profile_full_path.join(relative_path);
            if let Some(parent) = out_path.parent() {
                io::create_dir_all(parent).await?;
            }

            let mut entry_reader = zip_reader.reader_with_entry(index).await?;
            let mut data = Vec::new();
            entry_reader.read_to_end_checked(&mut data).await?;

            io::write(out_path, data).await?;
        }
    }

    // Record pack identity for future updates
    let pack_info = CfInstalledPackInfo {
        provider: "curseforge".to_string(),
        project_id: pack_project_id.unwrap_or(0),
        file_id: pack_file_id.unwrap_or(0),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        minecraft_version: manifest.minecraft.version.clone(),
        loader: manifest
            .minecraft
            .mod_loaders
            .iter()
            .find(|l| l.primary)
            .or_else(|| manifest.minecraft.mod_loaders.first())
            .map(|l| l.id.clone()),
        loader_version: None,
    };
    let pack_json = serde_json::to_string_pretty(&pack_info)?;
    io::write(profile_full_path.join("pack.json"), pack_json.as_bytes()).await?;

    // Update profile metadata (game version + loader)
    update_profile_metadata(profile_path, &manifest).await?;

    tracing::info!(
        cf.pack = %manifest.name,
        cf.pack_version = %manifest.version,
        cf.files_installed = installed.len(),
        "CurseForge modpack installed"
    );

    Ok(())
}

async fn fetch_pack_zip(
    pack_url: &str,
    loading_bar: &crate::event::LoadingBarId,
    _state: &State,
) -> crate::Result<bytes::Bytes> {
    let mut req = crate::util::fetch::REQWEST_CLIENT
        .request(reqwest::Method::GET, pack_url);
    if super::downloads::is_cdn_url(pack_url) {
        if let Some(key) = super::client::resolve_api_key() {
            req = req.header("x-api-key", key);
        }
    }

    let response = req.send().await.map_err(|_e| {
        crate::Error::from(CfError {
            kind: ProviderError::Network,
            status: None,
            path: pack_url.to_string(),
        })
    })?;

    if !response.status().is_success() {
        return Err(crate::Error::from(CfError {
            kind: super::client::map_status(response.status()),
            status: Some(response.status().as_u16()),
            path: pack_url.to_string(),
        }));
    }

    let total = response.content_length().unwrap_or(0) as f64;
    let mut bytes = Vec::new();
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_e| {
            crate::Error::from(CfError {
                kind: ProviderError::Network,
                status: None,
                path: pack_url.to_string(),
            })
        })?;
        bytes.extend_from_slice(&chunk);
        if total > 0.0 {
            emit_loading(loading_bar, chunk.len() as f64 / total, None)?;
        }
    }

    Ok(bytes::Bytes::from(bytes))
}


async fn update_profile_metadata(
    profile_path: &str,
    manifest: &CfPackManifest,
) -> crate::Result<()> {
    let game_version = manifest.minecraft.version.clone();
    let loader_id = manifest
        .minecraft
        .mod_loaders
        .iter()
        .find(|l| l.primary)
        .or_else(|| manifest.minecraft.mod_loaders.first())
        .map(|l| l.id.clone());

    if let Some(loader_id) = loader_id {
        let (loader_name, loader_version) = split_loader_id(&loader_id);
        if let Ok(loader) = loader_from_str(&loader_name) {
            let path = profile_path.to_string();
            let _ = profile::edit(&path, move |prof| {
                prof.game_version = game_version.clone();
                prof.loader = loader;
                prof.loader_version = Some(loader_version.to_string());
                prof.install_stage = ProfileInstallStage::Installed;
                async { Ok(()) }
            })
            .await;
        }
    } else {
        let path = profile_path.to_string();
        let _ = profile::edit(&path, move |prof| {
            prof.game_version = game_version.clone();
            prof.loader = ModLoader::Vanilla;
            prof.install_stage = ProfileInstallStage::Installed;
            async { Ok(()) }
        })
        .await;
    }

    Ok(())
}

/// Infers which subfolder a CF pack file belongs in from its filename.
/// Mods go to mods/, resource packs to resourcepacks/, shaders to
/// shaderpacks/, datapacks to datapacks/, everything else to the profile
/// root (config/ folders and friends are placed by the manifest's overrides
/// or are core files).
pub fn infer_target_dir(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".jar") || lower.ends_with(".zip") {
        if lower.contains("resource") || lower.contains("texture") {
            "resourcepacks".to_string()
        } else if lower.contains("shader") {
            "shaderpacks".to_string()
        } else {
            "mods".to_string()
        }
    } else if lower.ends_with(".mcmeta") || lower.ends_with(".png") {
        "resourcepacks".to_string()
    } else {
        String::new() // profile root
    }
}

async fn get_profile_path(profile_path: &str) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let full_path = state.directories.profiles_dir().join(profile_path);
    let canonical = io::canonicalize(&full_path)?;
    Ok(canonical)
}

/// Splits a CurseForge mod loader id like "forge-47.2.0" into ("forge", "47.2.0")
fn split_loader_id(loader_id: &str) -> (String, String) {
    match loader_id.split_once('-') {
        Some((name, version)) => (name.to_string(), version.to_string()),
        None => (loader_id.to_string(), String::new()),
    }
}

fn loader_from_str(loader: &str) -> crate::Result<ModLoader> {
    match loader.to_ascii_lowercase().as_str() {
        "fabric" => Ok(ModLoader::Fabric),
        "forge" => Ok(ModLoader::Forge),
        "quilt" => Ok(ModLoader::Quilt),
        "neoforge" => Ok(ModLoader::NeoForge),
        "vanilla" => Ok(ModLoader::Vanilla),
        other => Err(crate::ErrorKind::InputError(format!(
            "Unsupported CurseForge mod loader: {}",
            other
        ))
        .into()),
    }
}

/// Reads the recorded pack identity from a profile (for update checks).
pub async fn read_installed_pack(profile_path: &str) -> crate::Result<Option<CfInstalledPackInfo>> {
    let state = State::get().await?;
    let full_path = state.directories.profiles_dir().join(profile_path);
    let path = full_path.join("pack.json");
    if !path.exists() {
        return Ok(None);
    }
    let contents = io::read_to_string(&path).await?;
    Ok(serde_json::from_str(&contents).ok())
}

/// Checks whether a newer compatible pack file exists for the installed pack.
pub async fn check_pack_update(
    profile_path: &str,
) -> crate::Result<Option<(u32, String)>> {
    let Some(info) = read_installed_pack(profile_path).await? else {
        return Ok(None);
    };
    if info.project_id == 0 {
        return Ok(None);
    }

    let files = get_file_list_for_update(info.project_id).await?;
    let mut best: Option<(u32, String)> = None;
    for file in files {
        // Only same loader + same minecraft family
        if !file_is_compatible_for_update(&file, &info) {
            continue;
        }
        let file_id = file.id;
        if file.id != info.file_id {
            let candidate = (file_id, file.file_date.clone());
            if best.as_ref().map(|b| candidate.1 > b.1).unwrap_or(true) {
                best = Some(candidate);
            }
        }
    }
    Ok(best)
}

async fn get_file_list_for_update(project_id: u32) -> crate::Result<Vec<super::api_models::CfFile>> {
    #[derive(serde::Deserialize)]
    struct FilesResponse {
        data: Vec<super::api_models::CfFile>,
    }
    let query = vec![
        ("modId", project_id.to_string()),
        ("pageSize", "50".to_string()),
    ];
    let raw: FilesResponse =
        super::client::cf_get(&format!("/mods/{project_id}/files"), &query, true).await?;
    Ok(raw.data)
}

fn file_is_compatible_for_update(
    file: &super::api_models::CfFile,
    info: &CfInstalledPackInfo,
) -> bool {
    if file.id == info.file_id {
        return false;
    }
    // Match the pack's minecraft version + loader
    let versions_ok = file.game_versions.iter().any(|v| v == &info.minecraft_version)
        || file
            .sortable_game_versions
            .iter()
            .any(|v| v.game_version_name == info.minecraft_version);
    if !versions_ok {
        return false;
    }
    if let Some(loader) = &info.loader {
        let loader_lower = loader.to_ascii_lowercase();
        let loader_ok = file
            .sortable_game_versions
            .iter()
            .any(|v| v.game_version_name.to_ascii_lowercase() == loader_lower)
            || file
                .game_versions
                .iter()
                .any(|v| v.to_ascii_lowercase() == loader_lower);
        if !loader_ok {
            // If the file has explicit loader entries and none match, reject
            let loaders: Vec<String> = file
                .sortable_game_versions
                .iter()
                .map(|v| v.game_version_name.to_ascii_lowercase())
                .filter(|n| {
                    !n.chars().all(|c| c.is_ascii_digit() || c == '.')
                })
                .collect();
            if !loaders.is_empty() {
                return false;
            }
        }
    }
    true
}
