//! Human-readable rendering and the viewer document used by the TS timeline.

use kernelkite_core as kk;
use kk::compare::{Comparison, Summary};
use kk::model::ProfileBundle;
use serde::Serialize;

/// Format a byte count with a binary unit suffix.
pub fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut v = n as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{n} B")
    } else {
        format!("{v:.2} {}", UNITS[u])
    }
}

/// Render a [`Summary`] as an aligned text block.
pub fn summary_text(s: &Summary) -> String {
    let mut out = String::new();
    out.push_str(&format!("Profile summary: {} [{}]\n", s.label, s.backend));
    out.push_str(&format!("  duration      : {} ms\n", s.duration_ms));
    out.push_str(&format!("  frames        : {}\n", s.frames));
    out.push_str(&format!("  distinct pids : {}\n", s.distinct_pids));
    out.push_str(&format!("  cpu seconds   : {:.3}\n", s.cpu_seconds));
    out.push_str(&format!(
        "  peak rss      : {}\n",
        human_bytes(s.peak_rss_bytes)
    ));
    if let (Some(r), Some(w)) = (s.read_bytes, s.write_bytes) {
        out.push_str(&format!("  disk read     : {}\n", human_bytes(r)));
        out.push_str(&format!("  disk write    : {}\n", human_bytes(w)));
    } else {
        out.push_str("  disk io       : (not recorded)\n");
    }
    match (s.net_rx_bytes, s.net_tx_bytes) {
        (Some(rx), Some(tx)) => {
            out.push_str(&format!("  net rx        : {}\n", human_bytes(rx)));
            out.push_str(&format!("  net tx        : {}\n", human_bytes(tx)));
        }
        _ => out.push_str("  network       : (not sampled)\n"),
    }
    out
}

/// Render a [`Comparison`] as a before/after table.
pub fn comparison_text(c: &Comparison) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Comparison: {} -> {}\n\n",
        c.before.label, c.after.label
    ));
    out.push_str(&format!(
        "{:<16} {:>16} {:>16} {:>14} {:>10}\n",
        "metric", "before", "after", "change", "pct"
    ));
    out.push_str(&"-".repeat(76));
    out.push('\n');
    for d in &c.deltas {
        let pct = match d.pct_change {
            Some(p) => format!("{p:+.1}%"),
            None => "n/a".to_string(),
        };
        out.push_str(&format!(
            "{:<16} {:>16.3} {:>16.3} {:>+14.3} {:>10}\n",
            d.metric, d.before, d.after, d.abs_change, pct
        ));
    }
    out
}

/// Render the observed process tree as an indented outline.
pub fn tree_text(bundle: &ProfileBundle) -> String {
    let tree = bundle.process_tree();
    // Map pid -> comm from the latest observation for nice labels.
    let mut comm: std::collections::BTreeMap<i32, String> = std::collections::BTreeMap::new();
    for f in &bundle.frames {
        for s in f.processes.values() {
            comm.insert(s.pid, s.comm.clone());
        }
    }
    let all_pids: std::collections::BTreeSet<i32> = comm.keys().copied().collect();
    // Roots are pids whose parent is not itself an observed pid.
    let mut roots: Vec<i32> = all_pids
        .iter()
        .copied()
        .filter(|pid| {
            let ppid = bundle
                .frames
                .iter()
                .flat_map(|f| f.processes.values())
                .find(|s| s.pid == *pid)
                .map(|s| s.ppid)
                .unwrap_or(0);
            !all_pids.contains(&ppid)
        })
        .collect();
    roots.sort_unstable();

    let mut out = String::new();
    out.push_str(&format!("Process tree ({} pids)\n", all_pids.len()));
    for r in roots {
        render_node(&mut out, r, &tree, &comm, 0);
    }
    out
}

fn render_node(
    out: &mut String,
    pid: i32,
    tree: &std::collections::BTreeMap<i32, Vec<i32>>,
    comm: &std::collections::BTreeMap<i32, String>,
    depth: usize,
) {
    let name = comm.get(&pid).map(String::as_str).unwrap_or("?");
    out.push_str(&format!("{}{} {}\n", "  ".repeat(depth), pid, name));
    if let Some(children) = tree.get(&pid) {
        for c in children {
            if *c != pid {
                render_node(out, *c, tree, comm, depth + 1);
            }
        }
    }
}

// ---- Viewer document ------------------------------------------------------

/// A per-process series consumed by the TypeScript timeline viewer.
#[derive(Debug, Serialize)]
pub struct ViewerSeries {
    pub pid: i32,
    pub ppid: i32,
    pub comm: String,
    /// CPU utilisation percentage per frame (0..N*100 for multi-core).
    pub cpu_pct: Vec<f64>,
    /// Resident memory per frame in bytes.
    pub rss_bytes: Vec<u64>,
}

/// Top-level document handed to the viewer: shared time axis plus series.
#[derive(Debug, Serialize)]
pub struct ViewerDocument {
    pub label: String,
    pub backend: String,
    pub interval_ms: u64,
    pub t_ms: Vec<u64>,
    pub series: Vec<ViewerSeries>,
}

/// Build a [`ViewerDocument`] from a bundle by computing per-frame CPU% from
/// the delta of cumulative CPU ticks between adjacent frames.
pub fn viewer_document(bundle: &ProfileBundle) -> ViewerDocument {
    let ticks_per_sec = bundle.meta.clock_ticks_per_sec.max(1) as f64;
    let t_ms: Vec<u64> = bundle.frames.iter().map(|f| f.t_ms).collect();
    let pids = bundle.pids();

    let mut series = Vec::with_capacity(pids.len());
    for pid in pids {
        let mut cpu_pct = Vec::with_capacity(bundle.frames.len());
        let mut rss_bytes = Vec::with_capacity(bundle.frames.len());
        let mut ppid = 0;
        let mut comm = String::new();
        let mut prev_cpu: Option<u64> = None;
        let mut prev_t: Option<u64> = None;

        for f in &bundle.frames {
            if let Some(s) = f.processes.get(&pid) {
                ppid = s.ppid;
                comm = s.comm.clone();
                let cpu = s.utime_ticks + s.stime_ticks;
                let pct = match (prev_cpu, prev_t) {
                    (Some(pc), Some(pt)) if f.t_ms > pt => {
                        let dticks = cpu.saturating_sub(pc) as f64;
                        let dt_secs = (f.t_ms - pt) as f64 / 1000.0;
                        if dt_secs > 0.0 {
                            (dticks / ticks_per_sec) / dt_secs * 100.0
                        } else {
                            0.0
