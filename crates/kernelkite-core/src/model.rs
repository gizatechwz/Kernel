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
}

/// A single per-process measurement at one sampling instant.
///
/// Counters are cumulative where the kernel exposes them cumulatively
/// (CPU jiffies, I/O byte counts). Derived rates are computed at analysis
/// time so the raw bundle stays a faithful record of what was observed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessSample {
    /// Process id.
    pub pid: i32,
    /// Parent process id (0 if unknown / root).
    pub ppid: i32,
    /// Short command name (comm).
    pub comm: String,
    /// User-space CPU time in clock ticks since process start.
    pub utime_ticks: u64,
    /// Kernel-space CPU time in clock ticks since process start.
    pub stime_ticks: u64,
    /// Resident set size in bytes.
    pub rss_bytes: u64,
    /// Virtual memory size in bytes.
    pub vsize_bytes: u64,
    /// Number of threads.
    pub threads: u32,
    /// Cumulative bytes read (storage layer), if available.
    pub read_bytes: Option<u64>,
    /// Cumulative bytes written (storage layer), if available.
    pub write_bytes: Option<u64>,
    /// Number of open file descriptors, if available.
    pub open_fds: Option<u32>,
}

/// One timeline frame: every process observed at a single instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// Milliseconds since capture start.
    pub t_ms: u64,
    /// Samples keyed by pid for stable ordering and easy lookup.
    #[serde(with = "pid_map")]
    pub processes: BTreeMap<i32, ProcessSample>,
    /// Optional coarse network summary for this frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkSummary>,
}

impl Frame {
    pub fn new(t_ms: u64) -> Self {
