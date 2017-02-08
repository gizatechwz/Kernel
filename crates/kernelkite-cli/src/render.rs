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
