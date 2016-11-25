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
        threads: 1,
        read_bytes: Some(cpu * 10),
        write_bytes: Some(cpu * 2),
        open_fds: Some(5),
    }
}

fn make_bundle(label: &str, cpu_end: u64, rss: u64) -> ProfileBundle {
    let mut frames = Vec::new();
    for (i, cpu) in [0u64, cpu_end / 2, cpu_end].iter().enumerate() {
        let mut f = Frame::new(i as u64 * 100);
        f.insert(sample(1000, 1, "make", *cpu, rss));
        f.insert(sample(1001, 1000, "cc1", cpu / 2, rss / 2));
        frames.push(f);
    }
    ProfileBundle {
        meta: CaptureMeta {
            schema_version: BUNDLE_SCHEMA_VERSION,
            backend: Backend::Fixture,
            label: label.into(),
            command: Some(vec!["make".into()]),
            clock_ticks_per_sec: 100,
            page_size_bytes: 4096,
            interval_ms: 100,
            started_unix_secs: None,
            host_os: "test".into(),
            exit_code: Some(0),
        },
        frames,
    }
}

#[test]
fn bundle_json_roundtrip_is_stable() {
    let b = make_bundle("before", 200, 4096 * 100);
    let json1 = kk::to_json(&b).unwrap();
    let parsed = kk::from_json(&json1).unwrap();
    let json2 = kk::to_json(&parsed).unwrap();
    assert_eq!(json1, json2, "serialization must be deterministic");
