//! CurseForge fingerprint support.
//!
//! CurseForge exposes fingerprint-based project lookup which the app can use
//! for repair / reconciliation / missing-mod detection. The provider-neutral
//! concept is "resolve installed content by hash"; CurseForge implements its
//! side here (the API takes CF fingerprints, so a single-file lookup is done
//! per installed file).

use super::client::cf_post;
use serde::Deserialize;

/// Resolves a list of CurseForge file fingerprints to project/file matches.
pub async fn resolve_fingerprints(
    fingerprints: &[u64],
) -> crate::Result<Vec<FingerprintMatch>> {
    #[derive(Deserialize)]
    struct FingerprintResponse {
        is_cache_built: bool,
        exact_matches: Vec<ExactMatch>,
        partial_matches: Vec<PartialMatch>,
        installed_fingerprints: Vec<u64>,
        unmatched_fingerprints: Vec<u64>,
    }

    #[derive(Deserialize)]
    struct ExactMatch {
        id: u32,
        file: super::api_models::CfFile,
        latest_files: Vec<super::api_models::CfFile>,
    }

    #[derive(Deserialize)]
    struct PartialMatch {
        id: u32,
        file: super::api_models::CfFile,
        latest_files: Vec<super::api_models::CfFile>,
    }

    let body = serde_json::json!({ "fingerprints": fingerprints });
    let raw: FingerprintResponse = cf_post("/fingerprints", &body).await?;

    let mut results = Vec::new();
    for m in raw.exact_matches {
        results.push(FingerprintMatch {
            fingerprint: m.file.file_fingerprint.unwrap_or(0),
            project_id: m.file.mod_id,
            file_id: m.file.id,
            file_name: m.file.file_name.clone(),
            project_name: None,
            exact: true,
        });
    }
    for m in raw.partial_matches {
        results.push(FingerprintMatch {
            fingerprint: m.file.file_fingerprint.unwrap_or(0),
            project_id: m.file.mod_id,
            file_id: m.file.id,
            file_name: m.file.file_name.clone(),
            project_name: None,
            exact: false,
        });
    }
    Ok(results)
}

/// A single fingerprint match result.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintMatch {
    pub fingerprint: u64,
    pub project_id: u32,
    pub file_id: u32,
    pub file_name: String,
    pub project_name: Option<String>,
    pub exact: bool,
}

/// Computes the CurseForge fingerprint of a file's bytes. The CF fingerprint
/// is a modified FNV-1a over the first bytes of the file (the algorithm used
/// by the CurseForge launcher: hash of the file header).
///
/// NOTE: this is provided for completeness; the authoritative fingerprint is
/// returned by the API in the file model. When the API file metadata is
/// available (which it always is in our flow), prefer that value.
pub fn fingerprint_of_bytes(bytes: &[u8]) -> u64 {
    // CurseForge fingerprints use a specific variant; we implement the
    // documented FNV-1a (64-bit) over the first 4 KiB of the file.
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    let limit = bytes.len().min(4096);
    for byte in &bytes[..limit] {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable() {
        let data = b"hello curseforge world";
        let a = fingerprint_of_bytes(data);
        let b = fingerprint_of_bytes(data);
        assert_eq!(a, b);
        assert_ne!(a, fingerprint_of_bytes(b"different"));
    }
}
