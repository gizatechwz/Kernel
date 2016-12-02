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
