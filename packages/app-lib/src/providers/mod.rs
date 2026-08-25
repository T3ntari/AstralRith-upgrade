//! Content provider layer.
//!
//! Normalized provider models (`types`), provider-agnostic compatibility
//! helpers (`compatibility`) and provider implementations (`cf`, and
//! eventually `modrinth`). The UI consumes the normalized models.

pub mod compatibility;
pub mod errors;
pub mod provider;
pub mod types;

pub mod cf;
