//! Client profile management for module configurations.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientProfile {
    pub id: String,
    pub name: String,
    pub modules: serde_json::Value,
    pub keybinds: serde_json::Value,
    pub hud: serde_json::Value,
    pub render: serde_json::Value,
    pub version_compatibility: serde_json::Value,
    pub active: bool,
    pub created: i64,
    pub modified: i64,
}

pub async fn list_profiles() -> crate::Result<Vec<ClientProfile>> {
    let pool = crate::State::get().await?.pool.clone();
    let rows = sqlx::query(
        "SELECT id, name, modules, keybinds, hud, render, version_compatibility, active, created, modified FROM client_profiles ORDER BY created DESC"
    )
    .fetch_all(&pool)
    .await?;

    let profiles: Vec<ClientProfile> = rows.iter().filter_map(|row| {
        let id: String = row.try_get("id").ok()?;
        let name: String = row.try_get("name").ok()?;
        let modules: String = row.try_get("modules").unwrap_or_else(|_| "{}".to_string());
        let keybinds: String = row.try_get("keybinds").unwrap_or_else(|_| "{}".to_string());
        let hud: String = row.try_get("hud").unwrap_or_else(|_| "{}".to_string());
        let render: String = row.try_get("render").unwrap_or_else(|_| "{}".to_string());
        let vc: String = row.try_get("version_compatibility").unwrap_or_else(|_| "[]".to_string());
        let active: bool = row.try_get("active").unwrap_or(false);
        let created: i64 = row.try_get("created").unwrap_or(0);
        let modified: i64 = row.try_get("modified").unwrap_or(0);

        Some(ClientProfile {
            id,
            name,
            modules: serde_json::from_str(&modules).unwrap_or(serde_json::json!({})),
            keybinds: serde_json::from_str(&keybinds).unwrap_or(serde_json::json!({})),
            hud: serde_json::from_str(&hud).unwrap_or(serde_json::json!({})),
            render: serde_json::from_str(&render).unwrap_or(serde_json::json!({})),
            version_compatibility: serde_json::from_str(&vc).unwrap_or(serde_json::json!([])),
            active,
            created,
            modified,
        })
    }).collect();

    Ok(profiles)
}

pub async fn get_profile(id: &str) -> crate::Result<ClientProfile> {
    let profiles = list_profiles().await?;
    profiles.into_iter().find(|p| p.id == id).ok_or_else(|| {
        crate::ErrorKind::NoValueFor(format!("client profile {id}")).as_error()
    })
}

pub async fn create_profile(
    name: &str,
    modules: Option<&str>,
    keybinds: Option<&str>,
    hud: Option<&str>,
    render: Option<&str>,
    version_compatibility: Option<&str>,
) -> crate::Result<ClientProfile> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let pool = crate::State::get().await?.pool.clone();

    sqlx::query(
        "INSERT INTO client_profiles (id, name, modules, keybinds, hud, render, version_compatibility, active, created, modified) VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, $8, $9)"
    )
    .bind(&id)
    .bind(name)
    .bind(modules.unwrap_or("{}"))
    .bind(keybinds.unwrap_or("{}"))
    .bind(hud.unwrap_or("{}"))
    .bind(render.unwrap_or("{}"))
    .bind(version_compatibility.unwrap_or("[]"))
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    get_profile(&id).await
}

pub async fn update_profile(
    id: &str,
    name: Option<&str>,
    modules: Option<&str>,
    keybinds: Option<&str>,
    hud: Option<&str>,
    render: Option<&str>,
    version_compatibility: Option<&str>,
    active: Option<bool>,
) -> crate::Result<()> {
    let now = Utc::now().timestamp();
    let pool = crate::State::get().await?.pool.clone();

    // Get existing record to fill in blanks
    let existing = get_profile(id).await?;

    sqlx::query(
        "UPDATE client_profiles SET name = $1, modules = $2, keybinds = $3, hud = $4, render = $5, version_compatibility = $6, active = $7, modified = $8 WHERE id = $9"
    )
    .bind(name.unwrap_or(&existing.name))
    .bind(modules.unwrap_or(&serde_json::to_string(&existing.modules).unwrap_or_default()))
    .bind(keybinds.unwrap_or(&serde_json::to_string(&existing.keybinds).unwrap_or_default()))
    .bind(hud.unwrap_or(&serde_json::to_string(&existing.hud).unwrap_or_default()))
    .bind(render.unwrap_or(&serde_json::to_string(&existing.render).unwrap_or_default()))
    .bind(version_compatibility.unwrap_or(&serde_json::to_string(&existing.version_compatibility).unwrap_or_default()))
    .bind(active.unwrap_or(existing.active))
    .bind(now)
    .bind(id)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn delete_profile(id: &str) -> crate::Result<()> {
    let pool = crate::State::get().await?.pool.clone();
    sqlx::query("DELETE FROM client_profiles WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn duplicate_profile(id: &str) -> crate::Result<ClientProfile> {
    let existing = get_profile(id).await?;
    let new_name = format!("{} copy", existing.name);
    create_profile(
        &new_name,
        Some(&serde_json::to_string(&existing.modules).unwrap_or_default()),
        Some(&serde_json::to_string(&existing.keybinds).unwrap_or_default()),
        Some(&serde_json::to_string(&existing.hud).unwrap_or_default()),
        Some(&serde_json::to_string(&existing.render).unwrap_or_default()),
        Some(&serde_json::to_string(&existing.version_compatibility).unwrap_or_default()),
    ).await
}
