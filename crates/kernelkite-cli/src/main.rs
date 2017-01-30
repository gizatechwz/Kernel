//! kernelkite command-line interface.
//!
//! Subcommands:
//!   run <bundle.json> -- <cmd> [args...]   Live-profile a command (Linux /proc).
//!   replay <fixture.json> <bundle.json>    Replay a fixture into a bundle.
//!   summary <bundle.json>                  Print derived metrics for a bundle.
//!   compare <before.json> <after.json>     Before/after comparison report.
//!   tree <bundle.json>                     Print the observed process tree.
//!   viewer <bundle.json> [out.json]        Emit viewer-ready timeline JSON.
//!
//! Global flags: --interval-ms N, --max-frames N, --label L, --network,
//!               --json (machine-readable output where supported).

use kernelkite_core as kk;
use std::process::ExitCode;

mod args;
mod render;

use args::{Cli, Command};

fn main() -> ExitCode {
    let cli = match Cli::parse(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(msg) => {
            eprintln!("error: {msg}\n");
            eprint!("{}", args::USAGE);
            return ExitCode::from(2);
        }
    };

    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("kernelkite: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> kk::Result<ExitCode> {
    // Move the command out so the remaining `cli` can be borrowed for flags.
    let Cli {
