//! CurseForge integration: installing mods and modpacks into instances

use crate::event::emit::{emit_loading, init_or_edit_loading, init_loading};
use crate::event::LoadingBarType;
use crate::data::ModLoader;
use crate::profile;
use crate::state::State;
use crate::util::fetch::fetch_advanced;
use crate::util::io;
use async_zip::base::read::seek::ZipFileReader;
use reqwest::Method;
use serde::Deserialize;
use std::io::Cursor;
use std::path::{Component, PathBuf};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfApiResponse<T> {
    data: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfModpackManifest {
    files: Vec<CfManifestFile>,
    minecraft: CfMinecraft,
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default = "default_overrides")]
    overrides: String,
}

fn default_overrides() -> String {
    "overrides".to_string()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfManifestFile {
    project_id: u32,
    file_id: u32,
    #[serde(default)]
    required: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<CfModLoader>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfModLoader {
    id: String,
    #[serde(default)]
    primary: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfResolvedFile {
    id: u32,
    mod_id: u32,
    download_url: Option<String>,
    file_name: String,
    file_length: u64,
}

/// Installs a single mod file into an instance's mods folder
pub async fn install_mod(
    profile_path: &str,
    file_url: &str,
    file_name: &str,
    display_name: &str,
    file_length: u64,
) -> crate::Result<()> {
    let state = State::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::PackFileDownload {
            profile_path: profile_path.to_string(),
            pack_name: display_name.to_string(),
            icon: None,
            pack_version: "Latest".to_string(),
        },
        file_length.max(1) as f64,
        "Downloading mod...",
    )
    .await?;

    let result = download_to_mods(profile_path, file_url, file_name, file_length, &loading_bar, &state).await;

    emit_loading(&loading_bar, 0.0, None)?;

    result
}

async fn download_to_mods(
    profile_path: &str,
    file_url: &str,
    file_name: &str,
    file_length: u64,
    loading_bar: &crate::event::LoadingBarId,
    state: &State,
) -> crate::Result<()> {
    let bytes = fetch_advanced(
        Method::GET,
        file_url,
        None,
        None,
        None,
        Some((loading_bar, file_length as f64)),
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    let mods_dir = get_profile_path(profile_path).await?.join("mods");
    io::create_dir_all(&mods_dir).await?;
    io::write(mods_dir.join(file_name), bytes).await?;

    Ok(())
}

async fn get_profile_path(profile_path: &str) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let full_path = state.directories.profiles_dir().join(profile_path);
    let canonical = io::canonicalize(&full_path)?;
    Ok(canonical)
}

/// Installs a CurseForge modpack into an existing instance
///
/// Downloads the pack zip, parses its manifest, resolves all file download URLs
/// through the CurseForge API (using the provided API key), downloads every
/// required file into the instance's mods folder and extracts the overrides
/// folder on top of the instance.
pub async fn install_modpack(
    profile_path: &str,
    pack_url: &str,
    pack_name: &str,
    api_key: &str,
) -> crate::Result<()> {
    let state = State::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::PackDownload {
            profile_path: profile_path.to_string(),
            pack_name: pack_name.to_string(),
            icon: None,
            pack_id: None,
            pack_version: None,
        },
        1.0,
        "Downloading modpack...",
    )
    .await?;

    let result = install_modpack_inner(profile_path, pack_url, api_key, &loading_bar, &state).await;

    emit_loading(&loading_bar, 0.0, None)?;

    result
}

async fn install_modpack_inner(
    profile_path: &str,
    pack_url: &str,
    api_key: &str,
    loading_bar: &crate::event::LoadingBarId,
    state: &State,
) -> crate::Result<()> {
    // Download the pack zip
    let zip_bytes = fetch_advanced(
        Method::GET,
        pack_url,
        None,
        None,
        None,
        Some((loading_bar, 1.0)),
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

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
    entry_reader.read_to_string_checked(&mut manifest_string).await?;

    let manifest: CfModpackManifest = serde_json::from_str(&manifest_string)?;

    // Resolve all required files through the CurseForge API
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
        let bytes = fetch_advanced(
            Method::POST,
            "https://api.curseforge.com/v1/mods/files",
            None,
            Some(body),
            Some(("x-api-key", api_key)),
            None,
            &state.api_semaphore,
            &state.pool,
        )
        .await?;

        let mut files: CfApiResponse<Vec<CfResolvedFile>> = serde_json::from_slice(&bytes)?;

        // Resolve any files without a direct download URL
        for file in files.data.iter_mut() {
            if file.download_url.is_none() {
                let url = format!(
                    "https://api.curseforge.com/v1/mods/{}/files/{}/download-url",
                    file.mod_id, file.id
                );
                let response: CfApiResponse<String> =
                    fetch_json_with_key(&url, api_key, state).await?;
                file.download_url = Some(response.data);
            }
        }

        files.data
    };

    // Byte-accurate progress: the loading bar's total becomes the sum of all file sizes
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
            pack_id: None,
            pack_version: Some(manifest.version.clone()),
        },
        total_bytes,
        "Downloading modpack files...",
    )
    .await?;

    // Download each required file into the mods folder
    let mods_dir = get_profile_path(profile_path).await?.join("mods");
    io::create_dir_all(&mods_dir).await?;

    for file in &resolved {
        let Some(download_url) = &file.download_url else {
            continue;
        };

        let _ = fetch_advanced(
            Method::GET,
            download_url,
            None,
            None,
            None,
            Some((&loading_bar, file.file_length as f64)),
            &state.api_semaphore,
            &state.pool,
        )
        .await
        .map(|bytes| io::write(mods_dir.join(&file.file_name), bytes));
    }

    // Extract the overrides folder on top of the profile
    let overrides_prefix = format!("{}/", manifest.overrides.trim_start_matches('/'));
    let profile_full_path = get_profile_path(profile_path).await?;

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
        if relative_path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
        {
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

    // Update the profile metadata with the modpack's loader and game version
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
            let _ = profile::edit(&path, move |profile| {
                profile.game_version = game_version.clone();
                profile.loader = loader;
                profile.loader_version = Some(loader_version.to_string());
                async { Ok(()) }
            })
            .await;
        }
    }

    Ok(())
}

async fn fetch_json_with_key<T: serde::de::DeserializeOwned>(
    url: &str,
    api_key: &str,
    state: &State,
) -> crate::Result<T> {
    let bytes = fetch_advanced(
        Method::GET,
        url,
        None,
        None,
        Some(("x-api-key", api_key)),
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;

    Ok(serde_json::from_slice(&bytes)?)
}

/// Splits a CurseForge mod loader id like "forge-47.2.0" into ("forge", "47.2.0")
fn split_loader_id(loader_id: &str) -> (String, String) {
    match loader_id.split_once('-') {
        Some((name, version)) => (name.to_string(), version.to_string()),
        None => (loader_id.to_string(), String::new()),
    }
}

fn loader_from_str(loader: &str) -> crate::Result<ModLoader> {
    match loader {
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
