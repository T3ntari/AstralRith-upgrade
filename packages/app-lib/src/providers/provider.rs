//! Content provider contract.
//!
//! Both content providers (Modrinth and CurseForge) expose the same
//! normalized operations. The UI consumes `UnifiedProject` / `UnifiedFile`
//! models only; provider-specific API JSON stays behind the provider layer.

use crate::providers::types::{UnifiedFile, UnifiedProject};

/// The common provider surface implemented by each content provider.
///
/// This is documented as a trait-like contract. Rust 2021 doesn't allow
/// async fns in traits without `async_trait`; instead, each provider module
/// exposes these operations as free functions with identical signatures.
pub trait ContentProvider {
    /// Search projects with pagination.
    fn search_projects(
        query: &str,
        page: u32,
        page_size: u32,
    ) -> impl std::future::Future<Output = crate::Result<crate::providers::types::SearchPage<UnifiedProject>>>;

    /// Get a project by ID.
    fn get_project(
        id: &str,
    ) -> impl std::future::Future<Output = crate::Result<UnifiedProject>>;

    /// List a project's files (compatibility-filtered + sorted).
    fn get_compatible_files(
        project_id: &str,
        minecraft_version: &str,
        loader: Option<crate::providers::types::MinecraftLoader>,
        release_type: Option<&str>,
    ) -> impl std::future::Future<Output = crate::Result<Vec<UnifiedFile>>>;

    /// Resolve a file's download (CDN / manual-download aware).
    fn resolve_download(
        project_id: &str,
        file_id: &str,
    ) -> impl std::future::Future<Output = crate::Result<crate::providers::types::ResolvedDownload>>;
}
