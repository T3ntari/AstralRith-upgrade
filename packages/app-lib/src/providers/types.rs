//! Unified provider models shared by all content providers (Modrinth and
//! CurseForge). The UI consumes these normalized models so provider-specific
//! API JSON never leaks into components.

use serde::{Deserialize, Serialize};

/// Canonical content provider identifier
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Modrinth,
    CurseForge,
}

/// Canonical Minecraft loader identifiers
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum MinecraftLoader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
    Quilt,
    LiteLoader,
    Rift,
    LegacyFabric,
    Ornithe,
    Cauldron,
    Every,
    Unknown,
}

impl MinecraftLoader {
    pub fn as_str(&self) -> &'static str {
        match self {
            MinecraftLoader::Vanilla => "vanilla",
            MinecraftLoader::Fabric => "fabric",
            MinecraftLoader::Forge => "forge",
            MinecraftLoader::NeoForge => "neoforge",
            MinecraftLoader::Quilt => "quilt",
            MinecraftLoader::LiteLoader => "liteloader",
            MinecraftLoader::Rift => "rift",
            MinecraftLoader::LegacyFabric => "legacy-fabric",
            MinecraftLoader::Ornithe => "ornithe",
            MinecraftLoader::Cauldron => "cauldron",
            MinecraftLoader::Every => "every",
            MinecraftLoader::Unknown => "unknown",
        }
    }
}

/// Normalized category
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectCategory {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub parent_category_id: Option<String>,
    pub class_id: Option<String>,
    pub icon_url: Option<String>,
}

/// Normalized author
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectAuthor {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub url: Option<String>,
}

/// Normalized screenshot/gallery item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectScreenshot {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub ordering: i64,
}

/// Unified project model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnifiedProject {
    pub provider: Provider,
    pub id: String,
    pub slug: String,
    pub name: String,

    pub summary: Option<String>,
    pub description: Option<String>,

    pub icon_url: Option<String>,
    pub banner_url: Option<String>,

    pub author: Option<String>,
    pub authors: Vec<ProjectAuthor>,

    pub categories: Vec<ProjectCategory>,

    pub download_count: Option<u64>,
    pub rating: Option<f64>,

    pub website_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,

    pub screenshots: Vec<ProjectScreenshot>,

    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub released_at: Option<String>,

    pub project_type: String,

    /// Release type of the latest file ("release" | "beta" | "alpha" | "")
    pub latest_file_release_type: String,

    /// Provider-specific payload preserved for the backend (never shown raw)
    pub provider_project_id: Option<String>,
}

/// Dependency relation
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileDependencyRelation {
    Required,
    Optional,
    Embedded,
    Incompatible,
    Tool,
}

/// Normalized dependency
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileDependency {
    pub project_id: String,
    pub file_id: Option<String>,
    pub relation: FileDependencyRelation,
}

/// File hashes
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileHashes {
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub md5: Option<String>,
}

/// Unified file/version model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnifiedFile {
    pub provider: Provider,
    pub id: String,
    pub project_id: String,

    pub name: String,
    pub filename: String,

    pub version: Option<String>,
    pub game_versions: Vec<String>,
    pub loader_types: Vec<String>,

    pub release_type: String,

    pub published_at: Option<String>,
    pub download_count: Option<u64>,
    pub size: Option<u64>,

    pub download_url: Option<String>,
    pub hashes: FileHashes,

    pub dependencies: Vec<FileDependency>,

    pub is_server_pack: bool,
    pub server_pack_file_id: Option<String>,

    pub provider_file_id: Option<String>,
    /// Provider-specific JSON preserved for backend use (downloads, installs)
    pub provider_data: Option<serde_json::Value>,
}

/// A resolved download (after CDN/API resolution)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedDownload {
    pub url: String,
    pub file_name: String,
    pub file_length: Option<u64>,
    pub hash: Option<String>,
    pub requires_manual_download: bool,
    pub project_url: Option<String>,
}

/// Paginated search result envelope
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchPage<T> {
    pub results: Vec<T>,
    pub total_count: u64,
    pub has_next_page: bool,
}

/// Manual download state for files that cannot be fetched automatically
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManualDownloadInfo {
    pub project_url: String,
    pub project_id: String,
    pub file_id: String,
}
