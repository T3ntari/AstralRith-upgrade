//! Bot profile management.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub version: Option<String>,
    pub auth_type: String,
    pub username: String,
    pub script: Option<String>,
    pub plugins: serde_json::Value,
    pub status: String,
    pub created: i64,
    pub modified: i64,
}

pub async fn list_bots() -> crate::Result<Vec<BotProfile>> {
    let pool = crate::State::get().await?.pool.clone();
    let rows = sqlx::query(
        "SELECT id, name, host, port, version, auth_type, username, script, plugins, created, modified FROM bot_profiles ORDER BY created DESC"
    )
    .fetch_all(&pool)
    .await?;

    let bots: Vec<BotProfile> = rows.iter().filter_map(|row| {
        let id: String = row.try_get("id").ok()?;
        let name: String = row.try_get("name").ok()?;
        let host: String = row.try_get("host").ok()?;
        let port: i64 = row.try_get("port").unwrap_or(25565);
        let version: Option<String> = row.try_get("version").ok().flatten();
        let auth_type: String = row.try_get("auth_type").unwrap_or_else(|_| "offline".to_string());
        let username: String = row.try_get("username").unwrap_or_default();
        let script: Option<String> = row.try_get("script").ok().flatten();
        let plugins: String = row.try_get("plugins").unwrap_or_else(|_| "[]".to_string());
        let created: i64 = row.try_get("created").unwrap_or(0);
        let modified: i64 = row.try_get("modified").unwrap_or(0);

        Some(BotProfile {
            id,
            name,
            host,
            port,
            version,
            auth_type,
            username,
            script,
            plugins: serde_json::from_str(&plugins).unwrap_or(serde_json::json!([])),
            status: "disconnected".to_string(),
            created,
            modified,
        })
    }).collect();

    Ok(bots)
}

pub async fn get_bot(id: &str) -> crate::Result<BotProfile> {
    let bots = list_bots().await?;
    bots.into_iter().find(|b| b.id == id).ok_or_else(|| {
        crate::ErrorKind::NoValueFor(format!("bot profile {id}")).as_error()
    })
}

pub async fn create_bot(
    name: &str,
    host: &str,
    port: Option<i64>,
    version: Option<&str>,
    auth_type: Option<&str>,
    username: &str,
    script: Option<&str>,
    plugins: Option<&str>,
) -> crate::Result<BotProfile> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let pool = crate::State::get().await?.pool.clone();

    sqlx::query(
        "INSERT INTO bot_profiles (id, name, host, port, version, auth_type, username, script, plugins, created, modified) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(&id)
    .bind(name)
    .bind(host)
    .bind(port.unwrap_or(25565))
    .bind(version)
    .bind(auth_type.unwrap_or("offline"))
    .bind(username)
    .bind(script)
    .bind(plugins.unwrap_or("[]"))
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    get_bot(&id).await
}

pub async fn update_bot(
    id: &str,
    name: Option<&str>,
    host: Option<&str>,
    port: Option<i64>,
    version: Option<&str>,
    auth_type: Option<&str>,
    username: Option<&str>,
    script: Option<&str>,
    plugins: Option<&str>,
) -> crate::Result<()> {
    let now = Utc::now().timestamp();
    let existing = get_bot(id).await?;
    let pool = crate::State::get().await?.pool.clone();

    sqlx::query(
        "UPDATE bot_profiles SET name = $1, host = $2, port = $3, version = $4, auth_type = $5, username = $6, script = $7, plugins = $8, modified = $9 WHERE id = $10"
    )
    .bind(name.unwrap_or(&existing.name))
    .bind(host.unwrap_or(&existing.host))
    .bind(port.unwrap_or(existing.port))
    .bind(version.or(existing.version.as_deref()))
    .bind(auth_type.unwrap_or(&existing.auth_type))
    .bind(username.unwrap_or(&existing.username))
    .bind(script.or(existing.script.as_deref()))
    .bind(plugins.unwrap_or(&serde_json::to_string(&existing.plugins).unwrap_or_default()))
    .bind(now)
    .bind(id)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn delete_bot(id: &str) -> crate::Result<()> {
    let pool = crate::State::get().await?.pool.clone();
    sqlx::query("DELETE FROM bot_profiles WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn duplicate_bot(id: &str) -> crate::Result<BotProfile> {
    let existing = get_bot(id).await?;
    let new_name = format!("{} copy", existing.name);
    create_bot(
        &new_name,
        &existing.host,
        Some(existing.port),
        existing.version.as_deref(),
        Some(&existing.auth_type),
        &existing.username,
        existing.script.as_deref(),
        Some(&serde_json::to_string(&existing.plugins).unwrap_or_default()),
    ).await
}
