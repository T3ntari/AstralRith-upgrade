use crate::api::Result;
use theseus::skins::{self, SkinModelType, SkinRecord};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("skins")
        .invoke_handler(tauri::generate_handler![
            skins_list,
            skins_add,
            skins_delete,
            skins_update,
            skins_duplicate,
            skins_reorder,
            skins_mark_used,
            skins_validate,
        ])
        .build()
}

#[tauri::command]
pub async fn skins_list() -> Result<Vec<SkinRecord>> {
    Ok(theseus::skins::list_skins().await?)
}

/// Add a new skin from raw PNG bytes
#[tauri::command]
pub async fn skins_add(
    name: String,
    bytes: Vec<u8>,
    model_type: Option<String>,
    source: Option<String>,
    account_id: Option<String>,
    favorite: Option<bool>,
) -> Result<SkinRecord> {
    Ok(theseus::skins::add_skin(
        &name,
        &bytes,
        model_type
            .as_deref()
            .map(SkinModelType::from_str)
            .unwrap_or(SkinModelType::Classic),
        source
            .as_deref()
            .map(theseus::skins::SkinSource::from_str)
            .unwrap_or(theseus::skins::SkinSource::Local),
        account_id.as_deref(),
        favorite.unwrap_or(false),
    )
    .await?)
}

#[tauri::command]
pub async fn skins_delete(id: String) -> Result<()> {
    Ok(theseus::skins::delete_skin(&id).await?)
}

#[tauri::command]
pub async fn skins_update(
    id: String,
    name: Option<String>,
    favorite: Option<bool>,
    model_type: Option<String>,
    sort_order: Option<i64>,
    account_id: Option<String>,
) -> Result<SkinRecord> {
    Ok(theseus::skins::update_skin(
        &id,
        name.as_deref(),
        favorite,
        model_type.as_deref().map(SkinModelType::from_str),
        sort_order,
        account_id.as_deref(),
    )
    .await?)
}

#[tauri::command]
pub async fn skins_duplicate(id: String) -> Result<SkinRecord> {
    Ok(theseus::skins::duplicate_skin(&id).await?)
}

#[tauri::command]
pub async fn skins_reorder(ids: Vec<String>) -> Result<()> {
    let refs: Vec<String> = ids;
    Ok(theseus::skins::reorder_skins(&refs).await?)
}

#[tauri::command]
pub async fn skins_mark_used(id: String) -> Result<()> {
    Ok(theseus::skins::mark_used(&id).await?)
}

/// Validates and adds a PNG skin file. Returns the created record.
#[tauri::command]
pub async fn skins_validate(
    name: String,
    bytes: Vec<u8>,
) -> Result<SkinRecord> {
    let (w, h) = theseus::skins::validate_skin_png(&bytes)?;
    let skin_name = if name.is_empty() {
        format!("Imported skin {w}x{h}")
    } else {
        name
    };
    let record = theseus::skins::add_skin(
        &skin_name,
        &bytes,
        SkinModelType::Classic,
        theseus::skins::SkinSource::Imported,
        None,
        false,
    )
    .await?;
    Ok(record)
}
