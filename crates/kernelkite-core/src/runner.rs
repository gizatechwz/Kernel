//! Capture loop and live command runner.
//!
//! [`capture`] drives any [`Sampler`] on a fixed cadence and assembles a
//! [`ProfileBundle`]. [`run_and_profile`] additionally spawns a real child
//! process, samples while it runs, and records its exit code — the "live
//! developer-loop" mode. On Linux the runner defaults to profiling the child's
//! process subtree so build tools that fork compilers/linkers are captured
//! along with their parent.

use crate::error::Result;
use crate::model::{Backend, CaptureMeta, ProfileBundle, BUNDLE_SCHEMA_VERSION};
use crate::sampler::Sampler;
use std::time::{Duration, Instant};

/// Options controlling a capture session.
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    /// Human label stored in the bundle (e.g. "before").
    pub label: String,
    /// Milliseconds between frames.
    pub interval_ms: u64,
    /// Maximum number of frames to collect (safety bound for `capture`).
    pub max_frames: usize,
    /// Record wall-clock start time in the bundle metadata. Disable for
    /// deterministic fixtures.
    pub record_wall_clock: bool,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        CaptureOptions {
            label: "capture".into(),
            interval_ms: 100,
            max_frames: 100,
            record_wall_clock: true,
        }
    }
}

fn now_unix_secs() -> Option<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn base_meta(sampler: &dyn Sampler, opts: &CaptureOptions) -> CaptureMeta {
    let host = sampler.host_info();
    CaptureMeta {
        schema_version: BUNDLE_SCHEMA_VERSION,
        backend: sampler.backend(),
        label: opts.label.clone(),
        command: None,
        clock_ticks_per_sec: host.clock_ticks_per_sec,
        page_size_bytes: host.page_size_bytes,
        interval_ms: opts.interval_ms,
        started_unix_secs: if opts.record_wall_clock {
            now_unix_secs()
        } else {
            None
        },
        host_os: std::env::consts::OS.to_string(),
        exit_code: None,
    }
