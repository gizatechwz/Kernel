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
    assert_eq!(b, parsed);
}

#[test]
fn schema_mismatch_is_rejected() {
    let mut b = make_bundle("x", 100, 4096);
    b.meta.schema_version = 999;
    let json = kk::to_json(&b).unwrap();
    let err = kk::from_json(&json).unwrap_err();
    match err {
        kk::Error::SchemaMismatch { found, expected } => {
            assert_eq!(found, 999);
            assert_eq!(expected, BUNDLE_SCHEMA_VERSION);
        }
        other => panic!("expected SchemaMismatch, got {other:?}"),
    }
}

#[test]
fn summarize_computes_cpu_and_io_deltas() {
    let b = make_bundle("run", 200, 4096 * 100);
    let s = kk::summarize(&b);
    // pid 1000 cpu 0->200 ticks = 200; pid 1001 0->100 = 100; total 300 ticks.
    // at 100 ticks/sec that is 3.0 cpu seconds.
    assert!((s.cpu_seconds - 3.0).abs() < 1e-9, "cpu={}", s.cpu_seconds);
    assert_eq!(s.distinct_pids, 2);
    assert_eq!(s.frames, 3);
    assert_eq!(s.duration_ms, 200);
    // read_bytes = cpu*10 delta: pid1000 (2000-0)+ pid1001 (1000-0) = 3000.
    assert_eq!(s.read_bytes, Some(3000));
    assert_eq!(s.write_bytes, Some(600));
}

#[test]
fn compare_reports_expected_direction() {
    let before = make_bundle("before", 400, 4096 * 200);
    let after = make_bundle("after", 200, 4096 * 100);
    let cmp = kk::compare(&before, &after);
    let cpu = cmp
        .deltas
        .iter()
        .find(|d| d.metric == "cpu_seconds")
        .unwrap();
    assert!(cpu.after < cpu.before, "after should be faster");
    assert!(cpu.abs_change < 0.0);
    assert!(cpu.pct_change.unwrap() < 0.0);
}

#[test]
fn fixture_replay_is_deterministic() {
    let fixture = kk::Fixture {
        clock_ticks_per_sec: 100,
        page_size_bytes: 4096,
        frames: {
            let mut v = Vec::new();
            for i in 0..4u64 {
                let mut f = Frame::new(i * 50);
                f.insert(sample(42, 1, "cargo", i * 25, 1_000_000));
                v.push(f);
            }
            v
        },
    };
    let json = serde_json::to_string(&fixture).unwrap();

    // Replaying the same fixture twice yields identical bundles.
    let run_once = || {
        let fx = kk::Fixture::from_json(&json).unwrap();
        let sampler = kk::FixtureSampler::new(fx);
        let opts = kk::CaptureOptions {
            label: "replay".into(),
            interval_ms: 50,
            max_frames: 1000,
            record_wall_clock: false,
        };
        kk::capture(sampler, &opts).unwrap()
    };
    let a = run_once();
