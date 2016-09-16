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
}

/// Drive a sampler until it is exhausted, `max_frames` is reached, or (for
/// non-fixture backends) `interval_ms * max_frames` has elapsed.
///
/// For fixture backends this replays every recorded frame with no sleeping,
/// which keeps replay fast and deterministic.
pub fn capture(mut sampler: impl Sampler, opts: &CaptureOptions) -> Result<ProfileBundle> {
    let meta = base_meta(&sampler, opts);
    let is_fixture = sampler.backend() == Backend::Fixture;
    let mut frames = Vec::new();
    let start = Instant::now();

    for i in 0..opts.max_frames {
        let t_ms = if is_fixture {
            // Fixture replay ignores the hint; use index-derived time only as
            // a fallback for empty timestamps.
            (i as u64) * opts.interval_ms
        } else {
            start.elapsed().as_millis() as u64
        };
        match sampler.sample(t_ms)? {
            Some(frame) => frames.push(frame),
            None => break,
        }
        if !is_fixture && i + 1 < opts.max_frames {
            std::thread::sleep(Duration::from_millis(opts.interval_ms));
        }
    }

    Ok(ProfileBundle { meta, frames })
}

/// Spawn `command`, profile it with `sampler` until it exits, and return the
/// bundle including the child's exit code.
///
/// The sampler is expected to already be scoped to the interesting pids (for
/// example a `PidFilter::Subtree` rooted at the child). The child pid is
/// returned to the caller via `on_spawn` so a Linux caller can build such a
/// subtree filter *after* the fork.
pub fn run_and_profile<S, F>(
