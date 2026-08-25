//! Maps raw CurseForge API DTOs into the normalized provider models used by
//! the rest of the application.

use super::api_models::{
    CfCategory, CfFile, CfMod, CfModLoader,
};
use crate::providers::types::{
    FileDependency, FileDependencyRelation, FileHashes, MinecraftLoader,
    ProjectAuthor, ProjectCategory, ProjectScreenshot, Provider, UnifiedFile,
    UnifiedProject,
};

/// Release type: 1 = release, 2 = beta, 3 = alpha
pub fn release_type_name(release_type: u32) -> String {
    match release_type {
        1 => "release".to_string(),
        2 => "beta".to_string(),
        3 => "alpha".to_string(),
        other => format!("type{other}"),
    }
}

/// Maps a CurseForge relation type to the canonical dependency relation
pub fn relation_name(relation: u32) -> FileDependencyRelation {
    match relation {
        1 => FileDependencyRelation::Required,
        2 => FileDependencyRelation::Optional,
        3 => FileDependencyRelation::Embedded,
        4 => FileDependencyRelation::Incompatible,
        5 => FileDependencyRelation::Tool,
        _ => FileDependencyRelation::Optional,
    }
}

pub fn map_category(cat: &CfCategory) -> ProjectCategory {
    ProjectCategory {
        id: cat.id.to_string(),
        name: cat.name.clone(),
        slug: Some(cat.slug.clone()),
        parent_category_id: cat.parent_category_id.map(|x| x.to_string()),
        class_id: cat.class_id.map(|x| x.to_string()),
        icon_url: cat.icon_url.clone(),
    }
}

pub fn map_author(author: &super::api_models::CfAuthor) -> ProjectAuthor {
    ProjectAuthor {
        id: author.id.to_string(),
        name: author.name.clone(),
        avatar_url: None,
        url: Some(author.url.clone()),
    }
}

pub fn map_project(mod_: &CfMod, project_type: &str) -> UnifiedProject {
    let authors: Vec<ProjectAuthor> =
        mod_.authors.iter().map(map_author).collect();
    let author = mod_.authors.first().map(|a| a.name.clone());

    UnifiedProject {
        provider: Provider::CurseForge,
        id: mod_.id.to_string(),
        slug: mod_.slug.clone(),
        name: mod_.name.clone(),
        summary: Some(mod_.summary.clone()),
        description: None,
        icon_url: mod_.logo.as_ref().map(|l| l.url.clone()),
        banner_url: None,
        author,
        authors,
        categories: mod_.categories.iter().map(map_category).collect(),
        download_count: Some(mod_.download_count),
        rating: mod_.rating.or(mod_.thumbs_up_count.map(|t| t as f64)),
        website_url: mod_.links.website_url.clone(),
        source_url: mod_.links.source_url.clone(),
        issues_url: mod_.links.issues_url.clone(),
        wiki_url: mod_.links.wiki_url.clone(),
        screenshots: mod_
            .screenshots
            .iter()
            .enumerate()
            .map(|(i, s)| ProjectScreenshot {
                title: s.title.clone(),
                description: s.description.clone(),
                url: s.url.clone(),
                ordering: i as i64,
            })
            .collect(),
        created_at: Some(mod_.date_created.clone()),
        updated_at: Some(mod_.date_modified.clone()),
        released_at: Some(mod_.date_released.clone()),
        project_type: project_type.to_string(),
        latest_file_release_type: mod_
            .latest_files
            .first()
            .map(|f| release_type_name(f.release_type))
            .unwrap_or_default(),
        provider_project_id: Some(mod_.id.to_string()),
    }
}

/// Extracts loader names from a CF file's `gameVersions`/`sortableGameVersions`
pub fn extract_loaders(file: &CfFile) -> Vec<String> {
    let mut loaders: Vec<String> = Vec::new();
    for sortable in &file.sortable_game_versions {
        let name = sortable.game_version_name.to_lowercase();
        if is_loader_name(&name) {
            loaders.push(name);
        }
    }
    // Fallback: some files only list loaders inside gameVersions
    if loaders.is_empty() {
        for gv in &file.game_versions {
            let name = gv.to_lowercase();
            if is_loader_name(&name) {
                loaders.push(name);
            }
        }
    }
    loaders.sort();
    loaders.dedup();
    loaders
}

fn is_loader_name(name: &str) -> bool {
    matches!(
        name,
        "fabric"
            | "forge"
            | "neoforge"
            | "quilt"
            | "liteloader"
            | "rift"
            | "legacy-fabric"
            | "ornithe"
            | "cauldron"
            | "datapack"
            | "minecraft"
            | "client"
            | "server"
            | "bukkit"
            | "spigot"
            | "paper"
            | "purpur"
            | "folia"
    ) || name.starts_with("forge-")
        || name.starts_with("fabric-")
        || name.starts_with("neoforge-")
        || name.starts_with("quilt-")
        || name.starts_with("liteloader-")
}

/// The actual Minecraft game versions for a file (excludes loader entries)
pub fn extract_game_versions(file: &CfFile) -> Vec<String> {
    let mut versions: Vec<String> = Vec::new();
    for sortable in &file.sortable_game_versions {
        let name = sortable.game_version_name.to_lowercase();
        if !is_loader_name(&name) && !name.starts_with("forge-") {
            versions.push(sortable.game_version_name.clone());
        }
    }
    if versions.is_empty() {
        for gv in &file.game_versions {
            let name = gv.to_lowercase();
            if !is_loader_name(&name) && !name.starts_with("forge-") {
                versions.push(gv.clone());
            }
        }
    }
    versions.sort();
    versions.dedup();
    versions
}

pub fn map_file(file: &CfFile, _project_type: &str) -> UnifiedFile {
    let mut sha1 = None;
    let mut md5 = None;
    for hash in &file.hashes {
        match hash.algo {
            1 => sha1 = Some(hash.value.clone()),
            2 => md5 = Some(hash.value.clone()),
            _ => {}
        }
    }

    let dependencies: Vec<FileDependency> = file
        .dependencies
        .iter()
        .map(|d| FileDependency {
            project_id: d.mod_id.to_string(),
            file_id: None,
            relation: relation_name(d.relation_type),
        })
        .collect();

    let loaders = extract_loaders(file);
    let game_versions = extract_game_versions(file);

    UnifiedFile {
        provider: Provider::CurseForge,
        id: file.id.to_string(),
        project_id: file.mod_id.to_string(),
        name: file.display_name.clone(),
        filename: file.file_name.clone(),
        version: None,
        game_versions,
        loader_types: loaders,
        release_type: release_type_name(file.release_type),
        published_at: Some(file.file_date.clone()),
        download_count: Some(file.download_count),
        size: Some(file.file_length),
        download_url: file.download_url.clone(),
        hashes: FileHashes { sha1, sha512: None, md5 },
        dependencies,
        is_server_pack: file.is_server_pack,
        server_pack_file_id: file.server_pack_file_id.map(|x| x.to_string()),
        provider_file_id: Some(file.id.to_string()),
        provider_data: Some(serde_json::to_value(file).unwrap_or_default()),
    }
}

/// Canonical loader from a raw CF file (best effort, based on loader entries)
pub fn file_loader(file: &CfFile) -> MinecraftLoader {
    let loaders = extract_loaders(file);
    for loader in &loaders {
        let canonical = crate::providers::compatibility::loader_from_str(loader);
        if canonical != MinecraftLoader::Unknown
            && canonical != MinecraftLoader::Vanilla
        {
            return canonical;
        }
    }
    MinecraftLoader::Unknown
}

pub fn map_loader(loader: &CfModLoader) -> (u32, String, String) {
    (
        loader.mod_loader_type,
        loader.name.clone(),
        loader.game_version.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file() -> CfFile {
        serde_json::from_value(serde_json::json!({
            "id": 512345,
            "gameId": 432,
            "modId": 123,
            "isAvailable": true,
            "displayName": "Sodium 0.6.1",
            "fileName": "sodium-fabric-0.6.1+mc1.21.8.jar",
            "releaseType": 1,
            "fileStatus": 1,
            "hashes": [{"value": "abc123", "algo": 1}],
            "fileDate": "2026-08-23T00:00:00Z",
            "fileLength": 123456,
            "downloadCount": 42,
            "downloadUrl": "https://edge.forgecdn.net/files/1/2/3.jar",
            "gameVersions": ["1.21.8", "Fabric"],
            "sortableGameVersions": [
                {"gameVersionName": "1.21.8", "gameVersionPadded": "1.021.008", "gameVersion": "1.21.8", "gameVersionTypeId": 1},
                {"gameVersionName": "Fabric", "gameVersionPadded": "", "gameVersion": "", "gameVersionTypeId": 4}
            ],
            "dependencies": [{"modId": 999, "relationType": 1}],
            "alternateFileId": null,
            "isServerPack": false,
            "serverPackFileId": null,
            "fileFingerprint": 12345,
            "modules": null
        }))
        .unwrap()
    }

    #[test]
    fn maps_file() {
        let file = sample_file();
        let mapped = map_file(&file, "mod");
        assert_eq!(mapped.release_type, "release");
        assert_eq!(mapped.game_versions, vec!["1.21.8"]);
        assert!(mapped.loader_types.contains(&"fabric".to_string()));
        assert_eq!(mapped.hashes.sha1.as_deref(), Some("abc123"));
        assert_eq!(mapped.dependencies.len(), 1);
        assert_eq!(mapped.dependencies[0].relation, FileDependencyRelation::Required);
        assert_eq!(mapped.download_url.as_deref(), Some("https://edge.forgecdn.net/files/1/2/3.jar"));
        assert!(!mapped.is_server_pack);
    }
}
