//! Skin storage and management.
//!
//! Skins are stored as PNG files on disk (in the launcher config
//! `skins/` directory) with metadata in the SQLite database. The frontend
//! receives normalized `SkinRecord` models and the skin PNG bytes via the
//! asset protocol; the Rust side owns all filesystem access.
//!
//! Account-awareness: a skin can be bound to a Minecraft account (official
//! or offline). Offline accounts are explicitly distinguished and never
//! treated as Mojang-authenticated identities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::DirectoryInfo;
use crate::util::io;
use sqlx::Row;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkinModelType {
    Classic,
    Slim,
}

impl SkinModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkinModelType::Classic => "classic",
            SkinModelType::Slim => "slim",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "slim" => SkinModelType::Slim,
            _ => SkinModelType::Classic,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkinSource {
    Local,
    Imported,
    Edited,
    Account,
}

impl SkinSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkinSource::Local => "local",
            SkinSource::Imported => "imported",
            SkinSource::Edited => "edited",
            SkinSource::Account => "account",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "imported" => SkinSource::Imported,
            "edited" => SkinSource::Edited,
            "account" => SkinSource::Account,
            _ => SkinSource::Local,
        }
    }
}

/// Normalized skin record sent to the frontend.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinRecord {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub model_type: SkinModelType,
    pub source: SkinSource,
    pub account_id: Option<String>,
    pub favorite: bool,
    pub sort_order: i64,
    pub last_used: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    /// Asset URL usable by the frontend (asset protocol)
    pub asset_url: String,
}

/// The skins directory inside the launcher config dir
pub fn skins_dir() -> crate::Result<std::path::PathBuf> {
    let dir = DirectoryInfo::get_initial_settings_dir()
        .ok_or_else(|| {
            crate::ErrorKind::FSError(
                "Could not find valid settings dir".to_string(),
            )
        })?
        .join("skins");
    Ok(dir)
}

/// Validates a skin PNG: correct magic + sane dimensions (64x32 / 64x64).
pub fn validate_skin_png(bytes: &[u8]) -> crate::Result<(u32, u32)> {
    if bytes.len() < 8 {
        return Err(crate::ErrorKind::InputError(
            "Skin file is too small to be a PNG".to_string(),
        )
        .into());
    }
    // PNG magic
    if &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(crate::ErrorKind::InputError(
            "Skin file is not a valid PNG".to_string(),
        )
        .into());
    }
    // IHDR width/height at bytes 16-23 (big endian)
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width != 64 || (height != 32 && height != 64) {
        return Err(crate::ErrorKind::InputError(format!(
            "Unsupported skin dimensions: {width}x{height} (expected 64x32 or 64x64)"
        ))
        .into());
    }
    Ok((width, height))
}

/// Stores skin bytes to disk and creates a metadata record.
/// Returns the created SkinRecord.
pub async fn add_skin(
    name: &str,
    bytes: &[u8],
    model_type: SkinModelType,
    source: SkinSource,
    account_id: Option<&str>,
    favorite: bool,
) -> crate::Result<SkinRecord> {
    validate_skin_png(bytes)?;

    let dir = skins_dir()?;
    io::create_dir_all(&dir).await?;

    let id = Uuid::new_v4().to_string();
    let file_name = format!("{id}.png");
    io::write(dir.join(&file_name), bytes).await?;

    let now = Utc::now();
    let record = SkinRecord {
        id: id.clone(),
        name: name.to_string(),
        file_name: file_name.clone(),
        model_type,
        source,
        account_id: account_id.map(|s| s.to_string()),
        favorite,
        sort_order: 0,
        last_used: None,
        created: now,
        modified: now,
        asset_url: format!("asset://localhost/skins/{}", file_name),
    };

    upsert_record(&record).await?;
    Ok(record)
}

/// Lists all saved skins, ordered by sort order then created date.
pub async fn list_skins() -> crate::Result<Vec<SkinRecord>> {
    let pool = crate::State::get().await?.pool.clone();
    let rows = sqlx::query(
        "SELECT id, name, file_name, model_type, source, account_id, favorite, sort_order, last_used, created, modified FROM skins ORDER BY sort_order ASC, created ASC",
    )
    .fetch_all(&pool)
    .await?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row_to_record(&row)?);
    }
    Ok(records)
}

/// Deletes a skin by id (record + file).
pub async fn delete_skin(id: &str) -> crate::Result<()> {
    let pool = crate::State::get().await?.pool.clone();
    let rows = sqlx::query("SELECT file_name FROM skins WHERE id = $1")
        .bind(id)
        .fetch_all(&pool)
        .await?;
    if let Some(row) = rows.first() {
        let file_name: String = row.try_get("file_name")?;
        let dir = skins_dir()?;
        let path = dir.join(&file_name);
        if path.exists() {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    sqlx::query("DELETE FROM skins WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

/// Reads the PNG bytes of a skin by id.
pub async fn read_skin_bytes(id: &str) -> crate::Result<Vec<u8>> {
    let pool = crate::State::get().await?.pool.clone();
    let rows = sqlx::query("SELECT file_name FROM skins WHERE id = $1")
        .bind(id)
        .fetch_all(&pool)
        .await?;
    let row = rows.first().ok_or_else(|| {
        crate::ErrorKind::NoValueFor(format!("skin {id}")).as_error()
    })?;
    let file_name: String = row.try_get("file_name")?;
    let dir = skins_dir()?;
    Ok(io::read(&dir.join(&file_name)).await?)
}

/// Updates a skin's metadata fields.
pub async fn update_skin(
    id: &str,
    name: Option<&str>,
    favorite: Option<bool>,
    model_type: Option<SkinModelType>,
    sort_order: Option<i64>,
    account_id: Option<&str>,
) -> crate::Result<SkinRecord> {
    let pool = crate::State::get().await?.pool.clone();

    let rows = sqlx::query(
        "SELECT id, name, file_name, model_type, source, account_id, favorite, sort_order, last_used, created, modified FROM skins WHERE id = $1",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    let row = rows.first().ok_or_else(|| {
        crate::ErrorKind::NoValueFor(format!("skin {id}")).as_error()
    })?;
    let mut record = row_to_record(row)?;

    if let Some(name) = name {
        record.name = name.to_string();
    }
    if let Some(fav) = favorite {
        record.favorite = fav;
    }
    if let Some(mt) = model_type {
        record.model_type = mt;
    }
    if let Some(so) = sort_order {
        record.sort_order = so;
    }
    if let Some(acc) = account_id {
        record.account_id = Some(acc.to_string());
    } else if account_id.is_some() {
        // Option<&str> Some("") means clear
    }
    record.modified = Utc::now();
    upsert_record(&record).await?;
    Ok(record)
}

/// Marks a skin as used now (bumps last_used).
pub async fn mark_used(id: &str) -> crate::Result<()> {
    let pool = crate::State::get().await?.pool.clone();
    sqlx::query("UPDATE skins SET last_used = $1, modified = $2 WHERE id = $3")
        .bind(Utc::now().timestamp())
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

/// Duplicates a skin (new id, "copy" suffix, copied bytes).
pub async fn duplicate_skin(id: &str) -> crate::Result<SkinRecord> {
    let bytes = read_skin_bytes(id).await?;
    let pool = crate::State::get().await?.pool.clone();
    let rows = sqlx::query(
        "SELECT name, model_type, source, account_id, favorite FROM skins WHERE id = $1",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    let row = rows.first().ok_or_else(|| {
        crate::ErrorKind::NoValueFor(format!("skin {id}")).as_error()
    })?;
    let name: String = row.try_get("name")?;
    let model_type: String = row.try_get("model_type")?;
    let source: String = row.try_get("source")?;
    let account_id: Option<String> = row.try_get("account_id")?;
    let favorite: bool = row.try_get("favorite")?;

    add_skin(
        &format!("{name} copy"),
        &bytes,
        SkinModelType::from_str(&model_type),
        SkinSource::from_str(&source),
        account_id.as_deref(),
        favorite,
    )
    .await
}

/// Reorders skins by assigning sequential sort_order.
pub async fn reorder_skins(ids: &[String]) -> crate::Result<()> {
    let pool = crate::State::get().await?.pool.clone();
    for (index, id) in ids.iter().enumerate() {
        sqlx::query("UPDATE skins SET sort_order = $1, modified = $2 WHERE id = $3")
            .bind(index as i64)
            .bind(Utc::now().timestamp())
            .bind(id)
            .execute(&pool)
            .await?;
    }
    Ok(())
}

async fn upsert_record(record: &SkinRecord) -> crate::Result<()> {
    let pool = crate::State::get().await?.pool.clone();
    sqlx::query(
        "INSERT INTO skins (id, name, file_name, model_type, source, account_id, favorite, sort_order, last_used, created, modified)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (id) DO UPDATE SET
            name = $2, file_name = $3, model_type = $4, source = $5, account_id = $6,
            favorite = $7, sort_order = $8, last_used = $9, modified = $11",
    )
    .bind(&record.id)
    .bind(&record.name)
    .bind(&record.file_name)
    .bind(record.model_type.as_str())
    .bind(record.source.as_str())
    .bind(&record.account_id)
    .bind(record.favorite)
    .bind(record.sort_order)
    .bind(record.last_used.map(|d| d.timestamp()))
    .bind(record.created.timestamp())
    .bind(record.modified.timestamp())
    .execute(&pool)
    .await?;
    Ok(())
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> crate::Result<SkinRecord> {
    let id: String = row.try_get("id")?;
    let name: String = row.try_get("name")?;
    let file_name: String = row.try_get("file_name")?;
    let model_type: String = row.try_get("model_type")?;
    let source: String = row.try_get("source")?;
    let account_id: Option<String> = row.try_get("account_id")?;
    let favorite: bool = row.try_get("favorite")?;
    let sort_order: i64 = row.try_get("sort_order")?;
    let last_used: Option<i64> = row.try_get("last_used")?;
    let created: i64 = row.try_get("created")?;
    let modified: i64 = row.try_get("modified")?;

    let asset_url = format!("asset://localhost/skins/{}", file_name);
    Ok(SkinRecord {
        id,
        name,
        file_name,
        model_type: SkinModelType::from_str(&model_type),
        source: SkinSource::from_str(&source),
        account_id,
        favorite,
        sort_order,
        last_used: last_used.and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
        created: chrono::DateTime::from_timestamp(created, 0)
            .unwrap_or_else(Utc::now),
        modified: chrono::DateTime::from_timestamp(modified, 0)
            .unwrap_or_else(Utc::now),
        asset_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_png() {
        // Valid PNG header + IHDR 64x64
        let mut bytes = vec![0u8; 24];
        bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes[16..20].copy_from_slice(&64u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&64u32.to_be_bytes());
        assert!(validate_skin_png(&bytes).is_ok());

        // Bad magic
        let bad = b"not a png at all......";
        assert!(validate_skin_png(bad).is_err());

        // Bad dims
        let mut bad_dims = vec![0u8; 24];
        bad_dims[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        bad_dims[16..20].copy_from_slice(&128u32.to_be_bytes());
        bad_dims[20..24].copy_from_slice(&128u32.to_be_bytes());
        assert!(validate_skin_png(&bad_dims).is_err());
    }

    #[test]
    fn model_type_roundtrip() {
        assert_eq!(SkinModelType::from_str("slim"), SkinModelType::Slim);
        assert_eq!(SkinModelType::from_str("classic"), SkinModelType::Classic);
        assert_eq!(SkinModelType::from_str("nonsense"), SkinModelType::Classic);
    }
}
