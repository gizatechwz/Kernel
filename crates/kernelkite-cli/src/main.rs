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
        interval_ms,
        max_frames,
        label,
        network,
        json,
        command,
    } = cli;
    let flags = Cli {
        interval_ms,
        max_frames,
        label,
        network,
        json,
        command: Command::Help, // unused placeholder; only flags are read
    };
    match command {
        Command::Help => {
            print!("{}", args::USAGE);
            Ok(ExitCode::SUCCESS)
        }
        Command::Version => {
            println!("kernelkite {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Command::Run { out, command } => cmd_run(&flags, &out, &command),
        Command::Replay { fixture, out } => cmd_replay(&flags, &fixture, &out),
        Command::Summary { bundle } => cmd_summary(&flags, &bundle),
        Command::Compare { before, after } => cmd_compare(&flags, &before, &after),
        Command::Tree { bundle } => cmd_tree(&bundle),
        Command::Viewer { bundle, out } => cmd_viewer(&bundle, out.as_deref()),
    }
}

fn capture_opts(cli: &Cli, label: &str) -> kk::CaptureOptions {
    kk::CaptureOptions {
        label: cli.label.clone().unwrap_or_else(|| label.to_string()),
        interval_ms: cli.interval_ms,
        max_frames: cli.max_frames,
        record_wall_clock: true,
    }
}

fn cmd_run(cli: &Cli, out: &str, command: &[String]) -> kk::Result<ExitCode> {
    let opts = capture_opts(cli, "live");
    let with_network = cli.network;
    let bundle = kk::run_and_profile(
        command,
        |child_pid| {
            Ok(kk::ProcSampler::new(
                kk::PidFilter::Subtree(child_pid as i32),
                with_network,
            ))
        },
        &opts,
