//! Compatibility helpers: canonical loader mapping and semantic Minecraft
//! version comparison. Kept provider-agnostic and testable.

use super::types::MinecraftLoader;

/// Maps a CurseForge `modLoaderType` integer to the canonical loader.
pub fn loader_from_cf_id(id: u32) -> MinecraftLoader {
    match id {
        1 => MinecraftLoader::Forge,
        2 => MinecraftLoader::Cauldron,
        3 => MinecraftLoader::LiteLoader,
        4 => MinecraftLoader::Fabric,
        5 => MinecraftLoader::Quilt,
        6 => MinecraftLoader::NeoForge,
        7 => MinecraftLoader::Every,
        _ => MinecraftLoader::Unknown,
    }
}

/// Maps a canonical loader to the CurseForge `modLoaderType` integer.
pub fn loader_to_cf_id(loader: MinecraftLoader) -> u32 {
    match loader {
        MinecraftLoader::Forge => 1,
        MinecraftLoader::Cauldron => 2,
        MinecraftLoader::LiteLoader => 3,
        MinecraftLoader::Fabric => 4,
        MinecraftLoader::Quilt => 5,
        MinecraftLoader::NeoForge => 6,
        MinecraftLoader::Every => 7,
        _ => 0,
    }
}

/// Maps a string like "fabric", "neoforge" (as stored on profiles) to a
/// canonical loader.
pub fn loader_from_str(s: &str) -> MinecraftLoader {
    match s.to_ascii_lowercase().as_str() {
        "vanilla" => MinecraftLoader::Vanilla,
        "fabric" => MinecraftLoader::Fabric,
        "forge" => MinecraftLoader::Forge,
        "neoforge" | "neo" => MinecraftLoader::NeoForge,
        "quilt" => MinecraftLoader::Quilt,
        "liteloader" => MinecraftLoader::LiteLoader,
        "rift" => MinecraftLoader::Rift,
        "legacy-fabric" | "legacy_fabric" => MinecraftLoader::LegacyFabric,
        "ornithe" => MinecraftLoader::Ornithe,
        "cauldron" => MinecraftLoader::Cauldron,
        "every" => MinecraftLoader::Every,
        _ => MinecraftLoader::Unknown,
    }
}

/// The canonical loader name string, lower-case (same form as the profile
/// `loader` field in the app database).
pub fn loader_canonical_name(loader: MinecraftLoader) -> &'static str {
    match loader {
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

/// Parsed Minecraft version for semantic comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinecraftVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
    pub pre: Option<String>,
    pub raw: String,
}

/// Parses "1.21.8", "1.20.1", "1.21", "1.21.1-pre1", "24w14a", "1.8.9"
/// into a semantically comparable struct.
pub fn parse_minecraft_version(v: &str) -> Option<MinecraftVersion> {
    let v = v.trim();
    if v.is_empty() {
        return None;
    }

    // snapshots like 24w14a
    if v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        // e.g. "24w14a" -> digits then 'w'
        let digits: String = v.chars().take_while(|c| c.is_ascii_digit()).collect();
        let rest = &v[digits.len()..];
        if let Some(week) = rest.strip_prefix('w') {
            let week_num: String = week.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !week_num.is_empty() {
                let year: u32 = digits.parse().ok()?;
                let w: u32 = week_num.parse().ok()?;
                // snapshots sort after all releases; use a high synthetic number
                return Some(MinecraftVersion {
                    major: year,
                    minor: 900 + w,
                    patch: None,
                    pre: Some(format!("snapshot-{v}")),
                    raw: v.to_string(),
                });
            }
        }
    }

    // Split off pre-release suffix (e.g. -pre1, -rc1)
    let (core, pre) = match v.find(|c: char| c == '-' || c == '+') {
        Some(idx) => (&v[..idx], Some(v[idx..].to_string())),
        None => (v, None),
    };

    let mut parts = core.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch: Option<u32> = parts.next().and_then(|p| p.parse().ok());
    // Ignore extra components (e.g. 1.12.2-1.2.3 style won't occur here)

    Some(MinecraftVersion {
        major,
        minor,
        patch,
        pre,
        raw: v.to_string(),
    })
}

/// Semantic comparison of two Minecraft version strings.
/// Returns `std::cmp::Ordering`.
pub fn compare_minecraft_versions(a: &str, b: &str) -> std::cmp::Ordering {
    match (parse_minecraft_version(a), parse_minecraft_version(b)) {
        (Some(va), Some(vb)) => {
            let core_ordering = (va.major, va.minor, va.patch.unwrap_or(0))
                .cmp(&(vb.major, vb.minor, vb.patch.unwrap_or(0)));
            if core_ordering != std::cmp::Ordering::Equal {
                return core_ordering;
            }
            // Both same core version: pre-release sorts before release
            match (&va.pre, &vb.pre) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(pa), Some(pb)) => pa.cmp(pb),
            }
        }
        // Fall back to byte comparison when parsing fails; this is only for
        // exotic strings and keeps the function total.
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => a.cmp(b),
    }
}

/// Whether `candidate` equals or is a compatible release of `target`
/// (e.g. target "1.21" matches candidate "1.21.1"; target "1.21.8" only
/// matches "1.21.8").
pub fn version_matches(candidate: &str, target: &str) -> bool {
    if candidate == target {
        return true;
    }
    match (parse_minecraft_version(candidate), parse_minecraft_version(target)) {
        (Some(c), Some(t)) => {
            // Exact core match: 1.21.1 vs 1.21 (candidate newer patch of same minor)
            if c.major == t.major && c.minor == t.minor {
                if t.patch.is_none() {
                    return true; // target "1.21" matches any 1.21.x
                }
                return false; // 1.21.8 vs 1.21.1 -> not equal
            }
            false
        }
        _ => candidate == target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        assert_eq!(parse_minecraft_version("1.21.8").unwrap().patch, Some(8));
        assert_eq!(parse_minecraft_version("1.21").unwrap().patch, None);
        assert_eq!(parse_minecraft_version("1.20.1").unwrap().minor, 20);
        assert!(parse_minecraft_version("24w14a").is_some());
        assert!(parse_minecraft_version("1.8.9").is_some());
    }

    #[test]
    fn sorts_versions_semantically() {
        let mut versions = vec![
            "1.21.1", "1.21.10", "1.21.9", "1.21.8", "1.20.6", "1.20.4",
            "1.20.1", "1.19.4", "1.21.11", "1.21",
        ];
        versions.sort_by(|a, b| compare_minecraft_versions(a, b));
        assert_eq!(
            versions,
            vec![
                "1.19.4", "1.20.1", "1.20.4", "1.20.6", "1.21", "1.21.1",
                "1.21.8", "1.21.9", "1.21.10", "1.21.11",
            ]
        );
    }

    #[test]
    fn matches_compatible_versions() {
        assert!(version_matches("1.21.1", "1.21"));
        assert!(version_matches("1.21.8", "1.21.8"));
        assert!(!version_matches("1.21.8", "1.21.1"));
        assert!(!version_matches("1.20.4", "1.21"));
    }

    #[test]
    fn loader_mapping_roundtrip() {
        assert_eq!(loader_to_cf_id(MinecraftLoader::Fabric), 4);
        assert_eq!(loader_to_cf_id(MinecraftLoader::NeoForge), 6);
        assert_eq!(loader_from_cf_id(1), MinecraftLoader::Forge);
        assert_eq!(loader_from_str("neoforge"), MinecraftLoader::NeoForge);
    }
}
