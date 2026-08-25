use crate::api::Result;
use theseus::bots;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("bots")
        .invoke_handler(tauri::generate_handler![
            bot_list,
            bot_get,
            bot_create,
            bot_update,
            bot_delete,
            bot_duplicate,
        ])
        .build()
}

#[tauri::command]
pub async fn bot_list() -> Result<Vec<bots::BotProfile>> {
    Ok(bots::list_bots().await?)
}

#[tauri::command]
pub async fn bot_get(id: String) -> Result<bots::BotProfile> {
    Ok(bots::get_bot(&id).await?)
}

#[tauri::command]
pub async fn bot_create(
    name: String,
    host: String,
    port: Option<i64>,
    version: Option<String>,
    auth_type: Option<String>,
    username: String,
    script: Option<String>,
    plugins: Option<String>,
) -> Result<bots::BotProfile> {
    Ok(bots::create_bot(
        &name,
        &host,
        port,
        version.as_deref(),
        auth_type.as_deref(),
        &username,
        script.as_deref(),
        plugins.as_deref(),
    ).await?)
}

#[tauri::command]
pub async fn bot_update(
    id: String,
    name: Option<String>,
    host: Option<String>,
    port: Option<i64>,
    version: Option<String>,
    auth_type: Option<String>,
    username: Option<String>,
    script: Option<String>,
    plugins: Option<String>,
) -> Result<()> {
    Ok(bots::update_bot(
        &id,
        name.as_deref(),
        host.as_deref(),
        port,
        version.as_deref(),
        auth_type.as_deref(),
        username.as_deref(),
        script.as_deref(),
        plugins.as_deref(),
    ).await?)
}

#[tauri::command]
pub async fn bot_delete(id: String) -> Result<()> {
    Ok(bots::delete_bot(&id).await?)
}

#[tauri::command]
pub async fn bot_duplicate(id: String) -> Result<bots::BotProfile> {
    Ok(bots::duplicate_bot(&id).await?)
}
