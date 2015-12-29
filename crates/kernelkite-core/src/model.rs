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
        Frame {
            t_ms,
            processes: BTreeMap::new(),
            network: None,
        }
    }

    pub fn insert(&mut self, s: ProcessSample) {
        self.processes.insert(s.pid, s);
    }
}

/// Coarse, host-wide network counters. Sampled from `/proc/net/dev` on Linux.
/// This is intentionally a *summary* — kernelkite does not attribute traffic
/// to individual processes without an eBPF backend (which is not implemented).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSummary {
    /// Cumulative received bytes across sampled interfaces.
    pub rx_bytes: u64,
    /// Cumulative transmitted bytes across sampled interfaces.
    pub tx_bytes: u64,
    /// Interfaces that contributed to the summary.
    pub interfaces: Vec<String>,
}

/// Metadata describing how a bundle was captured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureMeta {
    pub schema_version: u32,
    pub backend: Backend,
    /// Label supplied by the user, e.g. "before" / "after" / "cold-cache".
    pub label: String,
    /// The command that was profiled, if a live runner was used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// Clock ticks per second on the capturing host (SC_CLK_TCK).
    pub clock_ticks_per_sec: u64,
    /// Page size in bytes on the capturing host.
    pub page_size_bytes: u64,
    /// Requested sampling interval in milliseconds.
    pub interval_ms: u64,
    /// Wall-clock start, seconds since the Unix epoch. `None` for fixtures so
    /// bundles stay byte-for-byte deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_unix_secs: Option<u64>,
    /// Host operating system family ("linux", "windows", "macos", ...).
    pub host_os: String,
    /// Exit code of the profiled command, if it terminated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

/// A complete, self-contained profile bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileBundle {
    pub meta: CaptureMeta,
    pub frames: Vec<Frame>,
}

impl ProfileBundle {
