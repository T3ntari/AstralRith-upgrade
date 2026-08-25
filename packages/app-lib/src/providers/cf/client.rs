//! CurseForge API client.
//!
//! All authenticated CurseForge API traffic happens here, on the Rust side.
//! The credential never crosses into frontend state, localStorage, logs,
//! URLs, or analytics.
//!
//! The key is resolved from infrastructure configuration:
//!   1. `CURSEFORGE_API_KEY` environment variable
//!   2. `AR_CF_API_KEY` environment variable (build/launch config)
//!   3. a `curseforge.key` file in the launcher config directory
//!   4. build-time compile flag (`CF_API_KEY` via cargo `--cfg`)
//!
//! If no key is configured the provider still functions for *public* API
//! routes that do not require authentication; authenticated routes return a
//! clean `ProviderError::AuthRequired` instead of leaking raw HTTP errors.

use crate::providers::errors::ProviderError;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

pub const CF_API_BASE: &str = "https://api.curseforge.com/v1";
pub const CF_GAME_ID: u32 = 432; // Minecraft
pub const CF_CDN_HOST: &str = "edge.forgecdn.net";
pub const CF_MAX_PAGE_SIZE: u32 = 50;
pub const CF_MAX_SEARCH_OFFSET: u32 = 10_000;

/// How many times a retryable failure is retried (in addition to the first
/// attempt). With exponential backoff + jitter this bounds total wait time.
const MAX_RETRIES: u32 = 4;

/// Resolves the application-level CurseForge credential from secure
/// infrastructure. Never logs the value.
pub fn resolve_api_key() -> Option<String> {
    if let Ok(key) = std::env::var("CURSEFORGE_API_KEY") {
        if !key.trim().is_empty() {
            return Some(key.trim().to_string());
        }
    }
    if let Ok(key) = std::env::var("AR_CF_API_KEY") {
        if !key.trim().is_empty() {
            return Some(key.trim().to_string());
        }
    }

    // Config file: <config>/curseforge.key (ignored by git)
    if let Some(config_dir) = crate::state::DirectoryInfo::get_initial_settings_dir() {
        let path = config_dir.join("curseforge.key");
        if let Ok(contents) = std::fs::read_to_string(&path) {
            let key = contents.trim();
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }

    // Build-time cfg flag: --cfg cf_api_key="..."
    #[cfg(cf_api_key)]
    {
        let key = cf_api_key!();
        if !key.is_empty() {
            return Some(key.to_string());
        }
    }

    None
}

fn retry_delay(attempt: u32) -> Duration {
    // exponential backoff with jitter: 500ms * 2^attempt + random 0..250ms
    let base = 500u64.saturating_mul(1u64 << attempt.min(6));
    let jitter = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() % 250_000_000)
        .unwrap_or(0)) as u64;
    Duration::from_millis(base + jitter / 1_000_000)
}

pub(crate) fn map_status(status: StatusCode) -> ProviderError {
    match status {
        StatusCode::UNAUTHORIZED => ProviderError::AuthRequired,
        StatusCode::FORBIDDEN => ProviderError::Forbidden,
        StatusCode::NOT_FOUND => ProviderError::NotFound,
        StatusCode::TOO_MANY_REQUESTS => ProviderError::RateLimited,
        StatusCode::REQUEST_TIMEOUT
        | StatusCode::GATEWAY_TIMEOUT
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::BAD_GATEWAY => ProviderError::Unavailable,
        StatusCode::INTERNAL_SERVER_ERROR => ProviderError::Unavailable,
        _ => ProviderError::Unknown,
    }
}

/// A normalized CurseForge API failure carrying structured context for
/// observability (never the credential).
#[derive(Debug)]
pub struct CfError {
    pub kind: ProviderError,
    pub status: Option<u16>,
    pub path: String,
}

impl std::fmt::Display for CfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CurseForge {} (status {:?}) for {}",
            self.kind, self.status, self.path
        )
    }
}

impl std::error::Error for CfError {}

impl From<CfError> for crate::Error {
    fn from(e: CfError) -> Self {
        crate::ErrorKind::OtherError(format!(
            "{}: {}",
            e.kind,
            e.kind.user_message()
        ))
        .into()
    }
}

/// Generic CurseForge API response envelope
#[derive(serde::Deserialize)]
pub struct CfResponse<T> {
    pub data: T,
}

/// Pagination block returned by the search endpoint
#[derive(serde::Deserialize, Debug, Clone)]
pub struct CfPagination {
    pub index: u32,
    pub page_size: u32,
    pub total_count: u32,
}

/// Full search response (raw + pagination)
#[derive(serde::Deserialize)]
pub struct CfSearchResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub pagination: Option<CfPagination>,
}

/// Performs a GET request against the CurseForge API with retry/backoff and
/// error normalization.
#[tracing::instrument(skip_all, fields(cf.path = %path))]
pub async fn cf_get<T: DeserializeOwned>(
    path: &str,
    query: &[(&str, String)],
    authenticated: bool,
) -> crate::Result<T> {
    let mut url = format!("{CF_API_BASE}{path}");
    let separator = if url.contains('?') { '&' } else { '?' };
    for (key, value) in query {
        if !value.is_empty() {
            url.push(separator);
            url.push_str(key);
            url.push('=');
            url.push_str(&urlencoding::encode(value));
        }
    }

    let key = if authenticated {
        match resolve_api_key() {
            Some(key) => key,
            None => {
                tracing::warn!(
                    cf.path = %path,
                    "CurseForge authenticated request attempted without a configured credential"
                );
                return Err(crate::Error::from(CfError {
                    kind: ProviderError::AuthRequired,
                    status: None,
                    path: path.to_string(),
                }));
            }
        }
    } else {
        String::new()
    };

    let start = std::time::Instant::now();
    let mut attempt = 0u32;
    loop {
        let mut req = crate::util::fetch::REQWEST_CLIENT.request(Method::GET, &url);
        if authenticated {
            req = req.header("x-api-key", &key);
        }
        req = req.header("Accept", "application/json");

        let result = req.send().await;
        match result {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            match serde_json::from_slice::<CfResponse<T>>(&bytes) {
                                Ok(parsed) => {
                                    tracing::debug!(
                                        cf.path = %path,
                                        cf.status = status.as_u16(),
                                        cf.duration_ms = start.elapsed().as_millis() as u64,
                                        "CurseForge request succeeded"
                                    );
                                    return Ok(parsed.data);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        cf.path = %path,
                                        "CurseForge returned invalid JSON: {}",
                                        e
                                    );
                                    return Err(crate::Error::from(CfError {
                                        kind: ProviderError::InvalidResponse,
                                        status: Some(status.as_u16()),
                                        path: path.to_string(),
                                    }));
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                cf.path = %path,
                                "Failed reading CurseForge response body: {}",
                                e
                            );
                            if attempt < MAX_RETRIES {
                                attempt += 1;
                                tokio::time::sleep(retry_delay(attempt)).await;
                                continue;
                            }
                            return Err(crate::Error::from(CfError {
                                kind: ProviderError::Network,
                                status: None,
                                path: path.to_string(),
                            }));
                        }
                    }
                } else {
                    let kind = map_status(status);
                    // Rate limits / transient server errors are retried with
                    // backoff. Auth errors are never retried (they will not
                    // succeed on retry).
                    let retryable = matches!(
                        kind,
                        ProviderError::RateLimited | ProviderError::Unavailable
                    );
                    if retryable && attempt < MAX_RETRIES {
                        tracing::debug!(
                            cf.path = %path,
                            cf.status = status.as_u16(),
                            cf.attempt = attempt + 1,
                            "CurseForge request failed, retrying"
                        );
                        attempt += 1;
                        tokio::time::sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    tracing::warn!(
                        cf.path = %path,
                        cf.status = status.as_u16(),
                        cf.error_kind = %kind,
                        "CurseForge request failed"
                    );
                    return Err(crate::Error::from(CfError {
                        kind,
                        status: Some(status.as_u16()),
                        path: path.to_string(),
                    }));
                }
            }
            Err(e) => {
                tracing::debug!(
                    cf.path = %path,
                    cf.attempt = attempt + 1,
                    "CurseForge network error: {}",
                    e
                );
                if attempt < MAX_RETRIES {
                    attempt += 1;
                    tokio::time::sleep(retry_delay(attempt)).await;
                    continue;
                }
                return Err(crate::Error::from(CfError {
                    kind: ProviderError::Network,
                    status: None,
                    path: path.to_string(),
                }));
            }
        }
    }
}

/// POST helper for batch endpoints (e.g. mods/files).
#[tracing::instrument(skip_all, fields(cf.path = %path))]
pub async fn cf_post<T: DeserializeOwned>(
    path: &str,
    body: &impl Serialize,
) -> crate::Result<T> {
    let key = resolve_api_key().ok_or_else(|| {
        crate::Error::from(CfError {
            kind: ProviderError::AuthRequired,
            status: None,
            path: path.to_string(),
        })
    })?;

    let url = format!("{CF_API_BASE}{path}");
    let start = std::time::Instant::now();
    let mut attempt = 0u32;
    loop {
        let mut req = crate::util::fetch::REQWEST_CLIENT
            .request(Method::POST, &url)
            .json(body);
        req = req.header("x-api-key", &key).header("Accept", "application/json");

        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.bytes().await {
                        Ok(bytes) => match serde_json::from_slice::<CfResponse<T>>(&bytes) {
                            Ok(parsed) => {
                                tracing::debug!(
                                    cf.path = %path,
                                    cf.status = status.as_u16(),
                                    cf.duration_ms = start.elapsed().as_millis() as u64,
                                    "CurseForge POST succeeded"
                                );
                                return Ok(parsed.data);
                            }
                            Err(e) => {
                                tracing::warn!("CurseForge POST invalid JSON: {}", e);
                                return Err(crate::Error::from(CfError {
                                    kind: ProviderError::InvalidResponse,
                                    status: Some(status.as_u16()),
                                    path: path.to_string(),
                                }));
                            }
                        },
                        Err(e) => {
                            tracing::warn!("Failed reading CurseForge POST body: {}", e);
                            if attempt < MAX_RETRIES {
                                attempt += 1;
                                tokio::time::sleep(retry_delay(attempt)).await;
                                continue;
                            }
                            return Err(crate::Error::from(CfError {
                                kind: ProviderError::Network,
                                status: None,
                                path: path.to_string(),
                            }));
                        }
                    }
                } else {
                    let kind = map_status(status);
                    let retryable = matches!(
                        kind,
                        ProviderError::RateLimited | ProviderError::Unavailable
                    );
                    if retryable && attempt < MAX_RETRIES {
                        attempt += 1;
                        tokio::time::sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    return Err(crate::Error::from(CfError {
                        kind,
                        status: Some(status.as_u16()),
                        path: path.to_string(),
                    }));
                }
            }
            Err(e) => {
                tracing::debug!("CurseForge POST network error: {}", e);
                if attempt < MAX_RETRIES {
                    attempt += 1;
                    tokio::time::sleep(retry_delay(attempt)).await;
                    continue;
                }
                return Err(crate::Error::from(CfError {
                    kind: ProviderError::Network,
                    status: None,
                    path: path.to_string(),
                }));
            }
        }
    }
}
