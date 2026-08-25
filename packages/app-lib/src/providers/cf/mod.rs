//! CurseForge provider: API client, mapping, compatibility, downloads and
//! modpack installation.
//!
//! The credential lives entirely on the Rust side (see `client::resolve_api_key`).
//! The frontend never sees, stores, or logs it.

pub mod api_models;
pub mod client;
pub mod downloads;
pub mod fingerprints;
pub mod mapper;
pub mod modpack;
pub mod service;

pub use client::{resolve_api_key, CfError, CF_API_BASE, CF_GAME_ID};
pub use service::{
    class_id_for_project_type, get_categories, get_compatible_files,
    get_file, get_game_versions, get_mod_loaders, get_project,
    get_project_by_slug, get_project_files, search_projects, select_best_file,
    CfSortField, SearchOptions,
};
pub use types_mapper::*;

mod types_mapper {
    pub use super::mapper::*;
}
