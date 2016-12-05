//! Small hand-rolled argument parser. Kept dependency-free so the CLI has a
//! minimal, auditable footprint and deterministic behaviour.

pub const USAGE: &str = "\
kernelkite — Linux-first local developer-loop process profiler

USAGE:
    kernelkite [GLOBAL FLAGS] <COMMAND> [ARGS]

COMMANDS:
    run <bundle.json> -- <cmd> [args...]   Live-profile a command via /proc (Linux).
    replay <fixture.json> <bundle.json>    Deterministically replay a fixture.
    summary <bundle.json>                  Print derived metrics for a bundle.
    compare <before.json> <after.json>     Before/after comparison report.
    tree <bundle.json>                     Print the observed process tree.
    viewer <bundle.json> [out.json]        Emit timeline JSON for the TS viewer.
    help                                   Show this help.
    version                                Show version.

GLOBAL FLAGS:
    --interval-ms <N>   Sampling interval in milliseconds (default 100).
    --max-frames <N>    Maximum frames to capture (default 100).
    --label <TEXT>      Label stored in the bundle (default per command).
    --network           Include a host-wide network summary (Linux /proc).
    --json              Emit machine-readable JSON where supported.

NOTES:
    The /proc backend is implemented and Linux-only. On other platforms use
    `replay` with a fixture. An eBPF backend is planned but NOT implemented;
    kernelkite never fabricates eBPF data.
";

/// Parsed command-line invocation.
#[derive(Debug, Clone)]
pub struct Cli {
    pub interval_ms: u64,
    pub max_frames: usize,
    pub label: Option<String>,
    pub network: bool,
    pub json: bool,
    pub command: Command,
}

/// The selected subcommand and its positional arguments.
#[derive(Debug, Clone)]
pub enum Command {
    Help,
    Version,
