use crate::api::Result;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("cf")
        .invoke_handler(tauri::generate_handler![
            cf_install_mod,
            cf_install_modpack
        ])
        .build()
}

// Installs a single CurseForge mod file into an instance's mods folder
// invoke('plugin:cf|cf_install_mod', { profilePath, fileUrl, fileName, displayName, fileLength })
#[tauri::command]
pub async fn cf_install_mod(
    profile_path: String,
    file_url: String,
    file_name: String,
    display_name: String,
    file_length: u64,
) -> Result<()> {
    Ok(theseus::cf::install_mod(
        &profile_path,
        &file_url,
        &file_name,
        &display_name,
        file_length,
    )
    .await?)
}

// Installs a CurseForge modpack into an existing instance
// invoke('plugin:cf|cf_install_modpack', { profilePath, packUrl, packName, apiKey })
#[tauri::command]
pub async fn cf_install_modpack(
    profile_path: String,
    pack_url: String,
    pack_name: String,
    api_key: String,
) -> Result<()> {
    Ok(theseus::cf::install_modpack(
        &profile_path,
        &pack_url,
        &pack_name,
        &api_key,
    )
    .await?)
}
