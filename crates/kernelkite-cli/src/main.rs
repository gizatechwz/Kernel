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
    )?;
    kk::save(&bundle, out)?;
    eprintln!(
        "captured {} frame(s) over {} ms -> {}",
        bundle.frames.len(),
        bundle.duration_ms(),
        out
    );
    let code = bundle.meta.exit_code.unwrap_or(0);
    Ok(ExitCode::from(code.clamp(0, 255) as u8))
}

fn cmd_replay(cli: &Cli, fixture: &str, out: &str) -> kk::Result<ExitCode> {
    let fx = kk::Fixture::load(fixture)?;
    let sampler = kk::FixtureSampler::new(fx);
    let mut opts = capture_opts(cli, "replay");
    // Fixture replay must be deterministic: drop wall-clock, allow all frames.
    opts.record_wall_clock = false;
    opts.max_frames = opts.max_frames.max(usize::MAX / 2);
    let bundle = kk::capture(sampler, &opts)?;
    kk::save(&bundle, out)?;
    eprintln!("replayed {} frame(s) -> {}", bundle.frames.len(), out);
    Ok(ExitCode::SUCCESS)
}

fn cmd_summary(cli: &Cli, bundle_path: &str) -> kk::Result<ExitCode> {
    let bundle = kk::load(bundle_path)?;
    let summary = kk::summarize(&bundle);
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print!("{}", render::summary_text(&summary));
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_compare(cli: &Cli, before: &str, after: &str) -> kk::Result<ExitCode> {
    let b = kk::load(before)?;
    let a = kk::load(after)?;
    let cmp = kk::compare(&b, &a);
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&cmp)?);
    } else {
        print!("{}", render::comparison_text(&cmp));
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_tree(bundle_path: &str) -> kk::Result<ExitCode> {
    let bundle = kk::load(bundle_path)?;
    print!("{}", render::tree_text(&bundle));
    Ok(ExitCode::SUCCESS)
}

fn cmd_viewer(bundle_path: &str, out: Option<&str>) -> kk::Result<ExitCode> {
    let bundle = kk::load(bundle_path)?;
    let doc = render::viewer_document(&bundle);
    let json = serde_json::to_string_pretty(&doc)?;
    match out {
        Some(path) => {
            std::fs::write(path, json)?;
            eprintln!("wrote viewer document -> {path}");
        }
        None => println!("{json}"),
    }
    Ok(ExitCode::SUCCESS)
# review note
