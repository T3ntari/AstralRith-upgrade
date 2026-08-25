//! Provider-aware error normalization.
//!
//! Both Modrinth and CurseForge failures map into this common format so the
//! UI can render meaningful, provider-aware messages instead of raw JS
//! exceptions.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderError {
    Network,
    AuthRequired,
    Forbidden,
    NotFound,
    RateLimited,
    Unavailable,
    NoDownload,
    Incompatible,
    InvalidResponse,
    Unknown,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ProviderError::Network => "network",
            ProviderError::AuthRequired => "auth_required",
            ProviderError::Forbidden => "forbidden",
            ProviderError::NotFound => "not_found",
            ProviderError::RateLimited => "rate_limited",
            ProviderError::Unavailable => "unavailable",
            ProviderError::NoDownload => "no_download",
            ProviderError::Incompatible => "incompatible",
            ProviderError::InvalidResponse => "invalid_response",
            ProviderError::Unknown => "unknown",
        };
        write!(f, "{s}")
    }
}

impl ProviderError {
    pub fn user_message(&self) -> String {
        match self {
            ProviderError::Network => "Network error while contacting CurseForge.".to_string(),
            ProviderError::AuthRequired => "CurseForge is temporarily unavailable (authentication required).".to_string(),
            ProviderError::Forbidden => "CurseForge rejected the request. The file may require manual download.".to_string(),
            ProviderError::NotFound => "The requested CurseForge content was not found.".to_string(),
            ProviderError::RateLimited => "CurseForge is rate limiting requests. Please wait a moment and try again.".to_string(),
            ProviderError::Unavailable => "CurseForge is temporarily unavailable.".to_string(),
            ProviderError::NoDownload => "This file requires manual download from CurseForge.".to_string(),
            ProviderError::Incompatible => "This file is not compatible with the selected Minecraft version or loader.".to_string(),
            ProviderError::InvalidResponse => "CurseForge returned an invalid response.".to_string(),
            ProviderError::Unknown => "An unexpected error occurred while contacting CurseForge.".to_string(),
        }
    }
}
