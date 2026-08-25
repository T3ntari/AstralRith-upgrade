//! CurseForge provider service: search, project metadata, file listing,
//! compatibility filtering and intelligent file selection. The UI consumes
//! normalized `UnifiedProject`/`UnifiedFile` models.

use super::api_models::{CfFile, CfGameVersion, CfMod, CfModLoader};
use super::client::{
    cf_get, CF_GAME_ID, CF_MAX_PAGE_SIZE, CF_MAX_SEARCH_OFFSET,
};
use super::mapper::{map_file, map_project};
use crate::providers::compatibility::{
    compare_minecraft_versions, loader_to_cf_id, version_matches,
};
use crate::providers::errors::ProviderError;
use crate::providers::types::{
    MinecraftLoader, SearchPage, UnifiedFile, UnifiedProject,
};
use serde::Deserialize;
use std::cmp::Ordering;

pub const CF_CLASS_MOD: u32 = 6;
pub const CF_CLASS_MODPACK: u32 = 4471;
pub const CF_CLASS_RESOURCEPACK: u32 = 12;
pub const CF_CLASS_SHADER: u32 = 6552;
pub const CF_CLASS_DATAPACK: u32 = 6945;

pub fn class_id_for_project_type(project_type: &str) -> u32 {
    match project_type {
        "modpack" => CF_CLASS_MODPACK,
        "resourcepack" => CF_CLASS_RESOURCEPACK,
        "shader" => CF_CLASS_SHADER,
        "datapack" => CF_CLASS_DATAPACK,
        _ => CF_CLASS_MOD,
    }
}

pub fn project_type_for_class(class_id: u32) -> &'static str {
    match class_id {
        CF_CLASS_MODPACK => "modpack",
        CF_CLASS_RESOURCEPACK => "resourcepack",
        CF_CLASS_SHADER => "shader",
        CF_CLASS_DATAPACK => "datapack",
        _ => "mod",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CfSortField {
    #[default]
    Popularity = 2,
    Featured = 1,
    LastUpdated = 3,
    Name = 4,
    Author = 5,
    TotalDownloads = 6,
    RecentlyUpdated = 7,
    RecentlyAdded = 8,
}

impl CfSortField {
    pub fn from_id(id: u32) -> Self {
        match id {
            1 => Self::Featured,
            3 => Self::LastUpdated,
            4 => Self::Name,
            5 => Self::Author,
            6 => Self::TotalDownloads,
            7 => Self::RecentlyUpdated,
            8 => Self::RecentlyAdded,
            _ => Self::Popularity,
        }
    }

    pub fn id(&self) -> u32 {
        *self as u32
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub query: String,
    pub page: u32,
    pub page_size: u32,
    pub sort_field: CfSortField,
    pub sort_order: String, // "asc" | "desc"
    pub game_version: String,
    pub mod_loader_type: u32,
    pub category_id: Option<u32>,
    pub class_id: u32,
}

/// Searches CurseForge mods with real pagination (index/pageSize).
/// The API caps page size at 50 and searchable range at 10,000.
pub async fn search_projects(
    options: &SearchOptions,
) -> crate::Result<SearchPage<UnifiedProject>> {
    let page_size = options.page_size.clamp(1, CF_MAX_PAGE_SIZE);
    let index = options
        .page
        .saturating_mul(page_size)
        .min(CF_MAX_SEARCH_OFFSET - page_size);

    let mut query: Vec<(&str, String)> = vec![
        ("gameId", CF_GAME_ID.to_string()),
        ("classId", options.class_id.to_string()),
        ("index", index.to_string()),
        ("pageSize", page_size.to_string()),
        ("sortField", options.sort_field.id().to_string()),
        ("sortOrder", options.sort_order.clone()),
    ];
    if !options.query.trim().is_empty() {
        query.push(("searchFilter", options.query.trim().to_string()));
    }
    if !options.game_version.is_empty() {
        query.push(("gameVersion", options.game_version.clone()));
    }
    if options.mod_loader_type != 0 {
        query.push(("modLoaderType", options.mod_loader_type.to_string()));
    }
    if let Some(category_id) = options.category_id {
        if category_id != 0 {
            query.push(("categoryId", category_id.to_string()));
        }
    }

    #[derive(Deserialize)]
    struct SearchResponse {
        data: Vec<CfMod>,
        #[serde(default)]
        pagination: Option<super::client::CfPagination>,
    }

    let raw: SearchResponse = cf_get("/mods/search", &query, true).await?;
    let project_type = project_type_for_class(options.class_id);
    let results: Vec<UnifiedProject> =
        raw.data.iter().map(|m| map_project(m, project_type)).collect();

    let total_count = raw
        .pagination
        .as_ref()
        .map(|p| p.total_count as u64)
        .unwrap_or_else(|| results.len() as u64);

    let has_next_page =
        ((index + page_size) as u64) < total_count.min(CF_MAX_SEARCH_OFFSET as u64);

    Ok(SearchPage {
        results,
        total_count,
        has_next_page,
    })
}

/// Fetches a single project by numeric ID.
pub async fn get_project(project_id: u32) -> crate::Result<UnifiedProject> {
    let raw: CfMod = cf_get(&format!("/mods/{project_id}"), &[], true).await?;
    let project_type = project_type_for_class(raw.class_id);
    Ok(map_project(&raw, project_type))
}

/// Fetches a project by slug (URL import support).
pub async fn get_project_by_slug(
    slug: &str,
    class_id: u32,
) -> crate::Result<UnifiedProject> {
    let mut query = vec![
        ("gameId", CF_GAME_ID.to_string()),
        ("slug", slug.to_string()),
    ];
    if class_id != 0 {
        query.push(("classId", class_id.to_string()));
    }

    #[derive(Deserialize)]
    struct ModsBySlug {
        data: Vec<CfMod>,
    }
    let raw: ModsBySlug = cf_get("/mods", &query, true).await?;
    let project = raw
        .data
        .into_iter()
        .next()
        .ok_or_else(|| crate::Error::from(super::client::CfError {
            kind: ProviderError::NotFound,
            status: Some(404),
            path: format!("/mods?slug={slug}"),
        }))?;
    let project_type = project_type_for_class(project.class_id);
    Ok(map_project(&project, project_type))
}

/// Lists files for a project, with optional server-side game version and
/// loader filtering (the API supports these natively).
pub async fn get_project_files(
    project_id: u32,
    game_version: Option<&str>,
    mod_loader_type: u32,
    page_size: u32,
) -> crate::Result<Vec<UnifiedFile>> {
    let page_size = page_size.clamp(1, CF_MAX_PAGE_SIZE);
    let mut query: Vec<(&str, String)> = vec![
        ("modId", project_id.to_string()),
        ("pageSize", page_size.to_string()),
    ];
    if let Some(gv) = game_version {
        if !gv.is_empty() {
            query.push(("gameVersion", gv.to_string()));
        }
    }
    if mod_loader_type != 0 {
        query.push(("modLoaderType", mod_loader_type.to_string()));
    }

    #[derive(Deserialize)]
    struct FilesResponse {
        data: Vec<CfFile>,
    }
    let raw: FilesResponse = cf_get(&format!("/mods/{project_id}/files"), &query, true).await?;
    let project_type = project_type_for_class(0); // class not known here; mod by default
    Ok(raw.data.iter().map(|f| map_file(f, project_type)).collect())
}

/// A single file lookup (by project + file id)
pub async fn get_file(project_id: u32, file_id: u32) -> crate::Result<UnifiedFile> {
    let raw: CfFile =
        cf_get(&format!("/mods/{project_id}/files/{file_id}"), &[], true).await?;
    Ok(map_file(&raw, "mod"))
}

/// Fetches the full file list and applies *client-side* compatibility
/// filtering + ordering. This is the function the version picker uses.
///
/// Ordering: compatible releases first (newest first), then compatible
/// betas/alphas, then incompatible files (visually demoted). Never just
/// `files[0]`.
pub async fn get_compatible_files(
    project_id: u32,
    minecraft_version: &str,
    loader: Option<MinecraftLoader>,
    release_type: Option<&str>,
    page_size: u32,
) -> crate::Result<Vec<UnifiedFile>> {
    let mut files = get_project_files(
        project_id,
        Some(minecraft_version),
        loader.map(loader_to_cf_id).unwrap_or(0),
        page_size.max(CF_MAX_PAGE_SIZE),
    )
    .await?;

    sort_and_annotate_files(&mut files, minecraft_version, loader, release_type);

    Ok(files)
}

/// Sorts files: compatible release > compatible beta > compatible alpha >
/// incompatible. Files are annotated via `is_compatible` on the frontend by
/// re-deriving from fields; the sort here is authoritative.
pub fn sort_and_annotate_files(
    files: &mut [UnifiedFile],
    minecraft_version: &str,
    loader: Option<MinecraftLoader>,
    release_type: Option<&str>,
) {
    let release_type = release_type.map(|s| s.to_ascii_lowercase());

    files.sort_by(|a, b| {
        let a_compat = file_is_compatible(a, minecraft_version, loader, release_type.as_deref());
        let b_compat = file_is_compatible(b, minecraft_version, loader, release_type.as_deref());
        match (a_compat, b_compat) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                let a_rel = release_rank(&a.release_type);
                let b_rel = release_rank(&b.release_type);
                let rel_cmp = a_rel.cmp(&b_rel);
                if rel_cmp != Ordering::Equal {
                    return rel_cmp;
                }
                // Newest first by publish date (ISO-8601 strings sort lexicographically)
                b.published_at
                    .clone()
                    .unwrap_or_default()
                    .cmp(&a.published_at.clone().unwrap_or_default())
            }
        }
    });
}

fn release_rank(release_type: &str) -> u8 {
    match release_type {
        "release" => 0,
        "beta" => 1,
        "alpha" => 2,
        _ => 3,
    }
}

/// Whether a unified CF file matches the given Minecraft version + loader +
/// release type constraints.
pub fn file_is_compatible(
    file: &UnifiedFile,
    minecraft_version: &str,
    loader: Option<MinecraftLoader>,
    release_type: Option<&str>,
) -> bool {
    // Release type
    if let Some(rt) = release_type {
        let rt = rt.to_ascii_lowercase();
        match rt.as_str() {
            "release" => {
                if file.release_type != "release" {
                    return false;
                }
            }
            "beta" => {
                if !matches!(file.release_type.as_str(), "release" | "beta") {
                    return false;
                }
            }
            "alpha" => {}
            _ => {}
        }
    }

    // Minecraft version
    let version_ok = file.game_versions.is_empty()
        || file.game_versions.iter().any(|v| version_matches(v, minecraft_version));
    if !version_ok {
        return false;
    }

    // Loader
    if let Some(loader) = loader {
        if loader == MinecraftLoader::Vanilla || loader == MinecraftLoader::Every {
            return true;
        }
        if !file.loader_types.is_empty() {
            let canonical: Vec<String> = file
                .loader_types
                .iter()
                .map(|l| crate::providers::compatibility::loader_from_str(l).as_str().to_string())
                .collect();
            let wanted = loader.as_str();
            if !canonical.iter().any(|l| l == wanted) {
                // allow "forge" files when loader is unknown (some packs)
                return false;
            }
        }
    }

    true
}

/// Finds the single best file for a project given the current instance
/// (Minecraft version + loader + release type). Used by "Install latest".
pub async fn select_best_file(
    project_id: u32,
    minecraft_version: &str,
    loader: Option<MinecraftLoader>,
    release_type: Option<&str>,
) -> crate::Result<UnifiedFile> {
    let files = get_compatible_files(
        project_id,
        minecraft_version,
        loader,
        release_type,
        CF_MAX_PAGE_SIZE,
    )
    .await?;

    files
        .into_iter()
        .find(|f| file_is_compatible(f, minecraft_version, loader, release_type))
        .ok_or_else(|| crate::Error::from(super::client::CfError {
            kind: ProviderError::Incompatible,
            status: None,
            path: format!("/mods/{project_id}/files (compatible selection)"),
        }))
}

/// Fetches and caches the category tree for a class (SQLite-backed with a
/// 24h TTL, same cache table used by the Modrinth side).
pub async fn get_categories(class_id: u32) -> crate::Result<Vec<super::api_models::CfCategory>> {
    if class_id == CF_CLASS_MOD {
        let state = crate::State::get().await?;
        if let Some(cached) = crate::state::CachedEntry::get_cf_categories(
            None,
            &state.pool,
            &state.api_semaphore,
        )
        .await?
        {
            return Ok(cached);
        }
    }

    let query = vec![
        ("gameId", CF_GAME_ID.to_string()),
        ("classId", class_id.to_string()),
    ];
    #[derive(Deserialize)]
    struct CategoriesResponse {
        data: Vec<super::api_models::CfCategory>,
    }
    let raw: CategoriesResponse = cf_get("/categories", &query, true).await?;
    Ok(raw.data)
}

/// Fetches Minecraft game versions from CurseForge (for the version picker),
/// cached for 24h.
pub async fn get_game_versions() -> crate::Result<Vec<String>> {
    let state = crate::State::get().await?;
    if let Some(cached) = crate::state::CachedEntry::get_cf_game_versions(
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    {
        return Ok(cached);
    }

    #[derive(Deserialize)]
    struct VersionsResponse {
        data: Vec<CfGameVersion>,
    }
    let raw: VersionsResponse = cf_get("/minecraft/version", &[], true).await?;
    let mut versions: Vec<String> = raw
        .data
        .iter()
        .filter(|v| {
            v.approved
                && !v.version_string.contains('_')
                && v.version_string
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.')
        })
        .map(|v| v.version_string.clone())
        .collect();
    versions.sort_by(|a, b| compare_minecraft_versions(a, b).reverse());
    versions.dedup();
    Ok(versions)
}

/// Fetches CurseForge mod loaders (for the loader picker), cached for 24h.
pub async fn get_mod_loaders() -> crate::Result<Vec<CfModLoader>> {
    let state = crate::State::get().await?;
    if let Some(cached) = crate::state::CachedEntry::get_cf_loaders(
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    {
        return cached
            .into_iter()
            .map(|v| serde_json::from_value(v).map_err(crate::Error::from))
            .collect();
    }

    #[derive(Deserialize)]
    struct LoadersResponse {
        data: Vec<CfModLoader>,
    }
    let raw: LoadersResponse = cf_get("/minecraft/modloader", &[], true).await?;
    Ok(raw.data)
}

/// Extracts unique loaders from a set of files (used by the version picker).
pub fn loaders_from_files(files: &[UnifiedFile]) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for file in files {
        for loader in &file.loader_types {
            if !set.contains(loader) {
                set.push(loader.clone());
            }
        }
    }
    set.sort();
    set
}

/// Extracts unique game versions from a set of files.
pub fn game_versions_from_files(files: &[UnifiedFile]) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for file in files {
        for version in &file.game_versions {
            if !set.contains(version) {
                set.push(version.clone());
            }
        }
    }
    set.sort_by(|a, b| compare_minecraft_versions(a, b).reverse());
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::types::Provider;

    fn file(release: &str, versions: &[&str], loaders: &[&str], date: &str) -> UnifiedFile {
        UnifiedFile {
            provider: Provider::CurseForge,
            id: "1".to_string(),
            project_id: "1".to_string(),
            name: "test".to_string(),
            filename: "test.jar".to_string(),
            version: None,
            game_versions: versions.iter().map(|s| s.to_string()).collect(),
            loader_types: loaders.iter().map(|s| s.to_string()).collect(),
            release_type: release.to_string(),
            published_at: Some(date.to_string()),
            download_count: Some(0),
            size: Some(1),
            download_url: None,
            hashes: Default::default(),
            dependencies: vec![],
            is_server_pack: false,
            server_pack_file_id: None,
            provider_file_id: None,
            provider_data: None,
        }
    }

    #[test]
    fn sorts_compatible_first() {
        let mut files = vec![
            file("release", &["1.21.1"], &["fabric"], "2025-01-01"),
            file("release", &["1.21.8"], &["fabric"], "2026-08-23"),
            file("beta", &["1.21.8"], &["fabric"], "2026-08-20"),
            file("release", &["1.21.8"], &["forge"], "2026-08-22"),
        ];
        sort_and_annotate_files(&mut files, "1.21.8", Some(MinecraftLoader::Fabric), None);
        // Compatible release first, then beta, then incompatible
        assert_eq!(files[0].game_versions, vec!["1.21.8".to_string()]);
        assert_eq!(files[0].release_type, "release");
        assert_eq!(files[1].release_type, "beta");
        assert!(files[2].release_type == "release" && files[2].loader_types == vec!["forge".to_string()]);
    }

    #[test]
    fn filters_release_type() {
        let mut files = vec![
            file("release", &["1.21.8"], &["fabric"], "2026-08-23"),
            file("beta", &["1.21.8"], &["fabric"], "2026-08-20"),
        ];
        sort_and_annotate_files(&mut files, "1.21.8", Some(MinecraftLoader::Fabric), Some("release"));
        assert!(files[0].release_type == "release");
        assert!(files[1].release_type != "release" || files[1].game_versions.is_empty());
    }

    #[test]
    fn rejects_wrong_loader() {
        let f = file("release", &["1.21.8"], &["forge"], "2026-08-23");
        assert!(!file_is_compatible(&f, "1.21.8", Some(MinecraftLoader::Fabric), None));
        assert!(file_is_compatible(&f, "1.21.8", Some(MinecraftLoader::Forge), None));
    }

    #[test]
    fn rejects_wrong_version() {
        let f = file("release", &["1.20.1"], &["forge"], "2026-08-23");
        assert!(!file_is_compatible(&f, "1.21.8", Some(MinecraftLoader::Forge), None));
    }
}
