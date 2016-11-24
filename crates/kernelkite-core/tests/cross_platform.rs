//! Cross-platform integration tests. These run on every OS, including Windows,
//! because they exercise the fixture, comparison, bundle and eBPF-stub paths
//! that do not touch `/proc`.

use kernelkite_core as kk;
use kk::model::{Backend, CaptureMeta, Frame, ProcessSample, ProfileBundle, BUNDLE_SCHEMA_VERSION};
use kk::sampler::Sampler;

fn sample(pid: i32, ppid: i32, comm: &str, cpu: u64, rss: u64) -> ProcessSample {
    ProcessSample {
        pid,
        ppid,
        comm: comm.into(),
        utime_ticks: cpu,
        stime_ticks: 0,
        rss_bytes: rss,
        vsize_bytes: rss * 4,
