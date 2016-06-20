//! Sampling abstraction.
//!
//! A [`Sampler`] produces one [`Frame`] per call to [`Sampler::sample`]. The
//! core ships three backends:
//!
//! * [`crate::proc_linux::ProcSampler`] — reads `/proc` on Linux (implemented).
//! * [`crate::fixture::FixtureSampler`] — replays recorded frames on any OS
//!   (implemented, deterministic).
//! * [`crate::ebpf::EbpfSampler`] — a stub for a FUTURE eBPF backend. It never
//!   fabricates data: every call returns [`Error::Unsupported`].

use crate::error::Result;
use crate::model::{Backend, Frame};

/// Host constants needed to interpret raw counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostInfo {
    /// SC_CLK_TCK — clock ticks per second (usually 100 on Linux).
    pub clock_ticks_per_sec: u64,
    /// Memory page size in bytes.
    pub page_size_bytes: u64,
}

impl Default for HostInfo {
    fn default() -> Self {
        HostInfo {
            clock_ticks_per_sec: 100,
            page_size_bytes: 4096,
        }
    }
}

impl HostInfo {
    /// Detect host constants. On non-Linux platforms this returns the
    /// conventional defaults, which is sufficient for fixture replay.
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            crate::proc_linux::detect_host_info()
        }
        #[cfg(not(target_os = "linux"))]
        {
            HostInfo::default()
