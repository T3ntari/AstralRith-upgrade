use crate::api::Result;
use theseus::client_profiles;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("client-profiles")
        .invoke_handler(tauri::generate_handler![
            client_profile_list,
            client_profile_get,
            client_profile_create,
            client_profile_update,
            client_profile_delete,
            client_profile_duplicate,
        ])
        .build()
}

#[tauri::command]
pub async fn client_profile_list() -> Result<Vec<client_profiles::ClientProfile>> {
    Ok(client_profiles::list_profiles().await?)
}

#[tauri::command]
pub async fn client_profile_get(id: String) -> Result<client_profiles::ClientProfile> {
    Ok(client_profiles::get_profile(&id).await?)
}

#[tauri::command]
pub async fn client_profile_create(
    name: String,
    modules: Option<String>,
    keybinds: Option<String>,
    hud: Option<String>,
    render: Option<String>,
    version_compatibility: Option<String>,
) -> Result<client_profiles::ClientProfile> {
    Ok(client_profiles::create_profile(
        &name,
        modules.as_deref(),
        keybinds.as_deref(),
        hud.as_deref(),
        render.as_deref(),
        version_compatibility.as_deref(),
    ).await?)
}

#[tauri::command]
pub async fn client_profile_update(
    id: String,
    name: Option<String>,
    modules: Option<String>,
    keybinds: Option<String>,
    hud: Option<String>,
    render: Option<String>,
    version_compatibility: Option<String>,
    active: Option<bool>,
) -> Result<()> {
    Ok(client_profiles::update_profile(
        &id,
        name.as_deref(),
        modules.as_deref(),
        keybinds.as_deref(),
        hud.as_deref(),
        render.as_deref(),
        version_compatibility.as_deref(),
        active,
    ).await?)
}

#[tauri::command]
pub async fn client_profile_delete(id: String) -> Result<()> {
    Ok(client_profiles::delete_profile(&id).await?)
}

#[tauri::command]
pub async fn client_profile_duplicate(id: String) -> Result<client_profiles::ClientProfile> {
    Ok(client_profiles::duplicate_profile(&id).await?)
}
