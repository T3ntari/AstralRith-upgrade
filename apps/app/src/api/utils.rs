use serde::{Deserialize, Serialize};
use theseus::{
    handler,
    prelude::{CommandPayload, DirectoryInfo},
};

use crate::api::Result;
use dashmap::DashMap;
use std::path::PathBuf;
use std::fs;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("utils")
        .invoke_handler(tauri::generate_handler![
            get_artifact,
            get_os,
            should_disable_mouseover,
            highlight_in_folder,
            open_path,
            show_launcher_logs_folder,
            progress_bars_list,
            get_opening_command,
            get_gpus
        ])
        .build()
}

#[tauri::command]
pub async fn get_artifact(downloadurl: &str, filename: &str, ostype: &str, autoupdatesupported: bool) -> Result<()> {
    theseus::download::init_download(downloadurl, filename, ostype, autoupdatesupported).await;
    Ok(())
}

/// Gets OS
#[tauri::command]
pub fn get_os() -> OS {
    #[cfg(target_os = "windows")]
    let os = OS::Windows;
    #[cfg(target_os = "linux")]
    let os = OS::Linux;
    #[cfg(target_os = "macos")]
    let os = OS::MacOS;
    os
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
}

// Lists active progress bars
// Create a new HashMap with the same keys
// Values provided should not be used directly, as they are not guaranteed to be up-to-date
#[tauri::command]
pub async fn progress_bars_list(
) -> Result<DashMap<uuid::Uuid, theseus::LoadingBar>> {
    let res = theseus::EventState::list_progress_bars().await?;
    Ok(res)
}

// cfg only on mac os
// disables mouseover and fixes a random crash error only fixed by recent versions of macos
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    // We try to match version to 12.2 or higher. If unrecognizable to pattern or lower, we default to the css with disabled mouseover for safety
    let os = os_info::get();
    if let os_info::Version::Semantic(major, minor, _) = os.version() {
        if *major >= 12 && *minor >= 3 {
            // Mac os version is 12.3 or higher, we allow mouseover
            return false;
        }
    }
    true
}
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    false
}

#[tauri::command]
pub fn highlight_in_folder(path: PathBuf) {
    let res = opener::reveal(path);

    if let Err(e) = res {
        tracing::error!("Failed to highlight file in folder: {}", e);
    }
}

#[tauri::command]
pub fn open_path(path: PathBuf) {
    let res = opener::open(path);

    if let Err(e) = res {
        tracing::error!("Failed to open path: {}", e);
    }
}

#[tauri::command]
pub fn show_launcher_logs_folder() {
    let path = DirectoryInfo::launcher_logs_dir().unwrap_or_default();
    // failure to get folder just opens filesystem
    // (ie: if in debug mode only and launcher_logs never created)
    open_path(path);
}

// Get opening command
// For example, if a user clicks on an .mrpack to open the app.
// This should be called once and only when the app is done booting up and ready to receive a command
// Returns a Command struct- see events.js
#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn get_opening_command(
    state: tauri::State<'_, crate::macos::deep_link::InitialPayload>,
) -> Result<Option<CommandPayload>> {
    let payload = state.payload.lock().await;

    return if let Some(payload) = payload.as_ref() {
        tracing::info!("opening command {payload}");

        Ok(Some(handler::parse_command(payload).await?))
    } else {
        Ok(None)
    };
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub async fn get_opening_command() -> Result<Option<CommandPayload>> {
    // Tauri is not CLI, we use arguments as path to file to call
    let cmd_arg = std::env::args_os().nth(1);

    tracing::info!("opening command {cmd_arg:?}");

    let cmd_arg = cmd_arg.map(|path| path.to_string_lossy().to_string());
    if let Some(cmd) = cmd_arg {
        tracing::debug!("Opening command: {:?}", cmd);
        return Ok(Some(handler::parse_command(&cmd).await?));
    }
    Ok(None)
}

// helper function called when redirected by a weblink (ie: modrith://do-something) or when redirected by a .mrpack file (in which case its a filepath)
// We hijack the deep link library (which also contains functionality for instance-checking)
pub async fn handle_command(command: String) -> Result<()> {
    tracing::info!("handle command: {command}");
    Ok(theseus::handler::parse_and_emit_command(&command).await?)
}

/// GPU information returned to the frontend
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuInfo {
    pub id: String,           // "auto" | "nvidia" | "integrated" | "dri:10de:1c82"
    pub name: String,         // Human-readable name
    pub vendor: String,       // "NVIDIA", "Intel", "AMD", etc.
    pub pci_id: Option<String>, // "10de:1c82"
    pub is_current: bool,     // Currently selected
}

/// List available GPUs for multi-GPU/PRIME GPU selection
#[tauri::command]
pub async fn get_gpus() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();

    // Always include "auto"
    gpus.push(GpuInfo {
        id: "auto".to_string(),
        name: "Automatic (system default)".to_string(),
        vendor: "System".to_string(),
        pci_id: None,
        is_current: false,
    });

    // Linux: scan /sys/class/drm for GPUs
    #[cfg(target_os = "linux")]
    {
        let drm_path = PathBuf::from("/sys/class/drm");
        if drm_path.exists() {
            if let Ok(entries) = fs::read_dir(&drm_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Look for card* directories (card0, card1, etc.)
                    if name.starts_with("card") && path.is_dir() {
                        let device_path = path.join("device");
                        if let Ok(vendor_id) = fs::read_to_string(device_path.join("vendor")) {
                            let vendor_id = vendor_id.trim();
                            let vendor_name = match vendor_id {
                                "0x10de" => "NVIDIA",
                                "0x8086" => "Intel",
                                "0x1002" => "AMD",
                                "0x15ad" => "VMware",
                                _ => "Unknown",
                            };
                            let pci_id = fs::read_to_string(device_path.join("uevent"))
                                .ok()
                                .and_then(|s| {
                                    s.lines()
                                        .find(|l| l.starts_with("PCI_ID="))
                                        .map(|l| l.strip_prefix("PCI_ID=").unwrap().to_string())
                                })
                                .unwrap_or_default();

                            let gpu_id = if vendor_name == "NVIDIA" {
                                format!("nvidia")
                            } else {
                                format!("integrated")
                            };
                            
                            // For non-NVIDIA, use DRI_PRIME with PCI ID
                            let gpu_id = if vendor_name == "NVIDIA" {
                                "nvidia".to_string()
                            } else if !pci_id.is_empty() {
                                format!("dri:{}", pci_id)
                            } else {
                                "integrated".to_string()
                            };

                            gpus.push(GpuInfo {
                                id: gpu_id,
                                name: format!("{} ({})", vendor_name, pci_id),
                                vendor: vendor_name.to_string(),
                                pci_id: if pci_id.is_empty() { None } else { Some(pci_id) },
                                is_current: false,
                            });
                        }
                    }
                }
            }
        }
    }

    // Mark the first non-auto as current if only one GPU (simplification)
    if gpus.len() == 2 {
        gpus[1].is_current = true;
    }

    Ok(gpus)
}
