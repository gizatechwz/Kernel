//! Data model for kernelkite profile bundles.
//!
//! Everything here is serializable so that a captured profile can be written
//! to disk as a deterministic JSON bundle and re-loaded by the CLI or the
//! TypeScript timeline viewer.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema version embedded in every bundle. Bumped on breaking changes.
pub const BUNDLE_SCHEMA_VERSION: u32 = 1;

/// Which sampling backend produced a bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    /// Linux `/proc` sampler (implemented).
    Proc,
    /// Deterministic fixture replay (cross-platform, implemented).
    Fixture,
    /// eBPF backend — reserved for the future, not implemented.
    Ebpf,
}

impl Backend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Backend::Proc => "proc",
            Backend::Fixture => "fixture",
            Backend::Ebpf => "ebpf",
        }
    }
