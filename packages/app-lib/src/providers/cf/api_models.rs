//! Raw CurseForge API models (DTOs). These mirror the official CurseForge
//! REST API JSON. The rest of the application consumes the normalized models
//! in `crate::providers::types` instead of these.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfMod {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub links: CfLinks,
    pub summary: String,
    pub status: u32,
    pub download_count: u64,
    pub is_featured: bool,
    pub primary_category_id: u32,
    pub categories: Vec<CfCategory>,
    pub class_id: u32,
    pub authors: Vec<CfAuthor>,
    pub logo: Option<CfImage>,
    pub screenshots: Vec<CfImage>,
    pub main_file_id: Option<u32>,
    pub latest_files: Vec<CfFile>,
    pub latest_files_indexes: Vec<CfFileIndex>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub allow_mod_distribution: Option<bool>,
    pub game_popularity_rank: Option<u32>,
    pub is_available: bool,
    pub thumbs_up_count: Option<u32>,
    pub rating: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfLinks {
    pub website_url: Option<String>,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfCategory {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub icon_url: Option<String>,
    pub date_modified: Option<String>,
    pub is_class: Option<bool>,
    pub class_id: Option<u32>,
    pub parent_category_id: Option<u32>,
    pub display_index: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfAuthor {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfImage {
    pub id: u32,
    pub mod_id: Option<u32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfFile {
    pub id: u32,
    pub game_id: u32,
    pub mod_id: u32,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: u32,
    pub file_status: u32,
    pub hashes: Vec<CfFileHash>,
    pub file_date: String,
    pub file_length: u64,
    pub download_count: u64,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
    pub sortable_game_versions: Vec<CfSortableGameVersion>,
    pub dependencies: Vec<CfFileDependency>,
    pub alternate_file_id: Option<u32>,
    pub is_server_pack: bool,
    pub server_pack_file_id: Option<u32>,
    pub file_fingerprint: Option<u64>,
    pub modules: Option<Vec<CfModule>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfFileHash {
    pub value: String,
    pub algo: u32, // 1 = SHA1, 2 = MD5
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfSortableGameVersion {
    pub game_version_name: String,
    pub game_version_padded: String,
    pub game_version: String,
    pub game_version_type_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfFileDependency {
    pub mod_id: u32,
    pub relation_type: u32, // 1 = required, 2 = optional, 3 = embedded, 4 = incompatible, 5 = tool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfModule {
    pub name: String,
    pub fingerprint: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfFileIndex {
    pub game_version: String,
    pub file_id: u32,
    pub filename: String,
    pub release_type: u32,
    pub game_version_type_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfGameVersion {
    pub id: u32,
    pub game_version_id: u32,
    pub version_string: String,
    pub jar_download_url: Option<String>,
    pub json_download_url: Option<String>,
    pub approved: bool,
    pub date_modified: String,
    pub game_version_type_id: u32,
    pub game_version_status: u32,
    pub game_version_type_status: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfModLoader {
    pub id: u32,
    pub game_version_id: u32,
    pub name: String,
    pub mod_loader_type: u32,
    pub download_url: Option<String>,
    pub filename: Option<String>,
    pub latest: bool,
    pub recommended: bool,
    pub approved: bool,
    pub date_modified: String,
    pub game_version: String,
}

/// A modpack manifest (manifest.json inside a CurseForge pack zip)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfPackManifest {
    pub minecraft: CfPackMinecraft,
    pub manifest_type: String,
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub files: Vec<CfPackFile>,
    pub overrides: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfPackMinecraft {
    pub version: String,
    pub mod_loaders: Vec<CfPackModLoader>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfPackModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfPackFile {
    pub project_id: u32,
    pub file_id: u32,
    pub required: bool,
}

/// Minimal resolved file info returned by POST /mods/files
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CfResolvedFile {
    pub id: u32,
    pub mod_id: u32,
    pub download_url: Option<String>,
    pub file_name: String,
    pub file_length: u64,
    pub hashes: Option<Vec<CfFileHash>>,
}
