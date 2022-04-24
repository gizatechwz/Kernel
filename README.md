<!-- kernelkite README -->

<p align="center">
  <img src="docs/assets/kernel-kite.svg" alt="kernelkite: sample a process subtree from /proc and land a deterministic bundle" width="880"/>
</p>

# KernelKite

A local, Linux-first profiler for the inner developer loop. You point it at a command you already run (`make`, `cargo`, `pytest`, `npm`, a shell script), it samples the whole process subtree from `/proc` on a fixed cadence, and it writes a deterministic bundle you can summarize, diff, replay, and render.

The design goal is narrow on purpose: measure what your machine actually did during `edit, build, test, repeat`, keep every number traceable to a `/proc` field, and produce output that is byte-for-byte reproducible so before/after comparisons mean something.

## What an operator can do

- Profile a live command and capture its process subtree (`run`, Linux).
- Replay a recorded fixture into a bundle on any OS (`replay`).
- Print derived metrics for a bundle (`summary`).
- Produce a before/after delta report (`compare`).
- Reconstruct the observed parent/child process tree (`tree`).
- Emit a viewer document and render a self-contained timeline (`viewer` plus the TypeScript renderer).

## How a capture flows

The launched command forks a subtree. On every interval kernelkite snapshots each in-scope process into a frame. A bundle is the ordered list of frames plus the host constants used to record them. Derived metrics are computed from counter deltas at analysis time, never baked into the capture.

Because a fixture records its own timestamps and omits wall-clock time, the same fixture always produces the same bytes. The entire `samples/` directory is generated from the shipped fixtures, and CI fails if it drifts.

## Backends

| Backend   | Status          | Platform   | Behavior                                            |
|-----------|-----------------|------------|-----------------------------------------------------|
| `proc`    | implemented     | Linux only | Samples `/proc` on every tick.                      |
| `fixture` | implemented     | any OS     | Replays a recorded timeline, deterministically.     |
| `ebpf`    | stub, not built | none       | Reserved. Returns `Error::Unsupported`, never fakes data. |

The `ebpf` type exists so the enum, CLI, and roadmap describe the intended shape honestly. It loads no BPF program and every call returns an error. See [`crates/kernelkite-core/src/ebpf.rs`](crates/kernelkite-core/src/ebpf.rs).

The `proc` backend is Linux-only, so the crate uses `cfg` guards to keep compiling on Windows and macOS, where the sampler becomes a compile-time shim that reports it needs Linux. On those hosts you use `fixture` replay to develop, demo, and test the whole tool.

## Install and build

Requirements: a stable Rust toolchain (1.74+) and, for the viewer, Node 18+.

```bash
git clone https://github.com/Mujung/KernelKite
cd kernelkite

cargo build --release          # -> target/release/kernelkite
cd viewer && npm install && npm run build && cd ..
```

The Makefile wraps the common targets:

```bash
make build      # release build of the workspace
make test       # full Rust test suite
make viewer     # build + test the TS viewer
make ci         # everything CI runs
make demo       # summary + comparison from the shipped samples
```

## Usage walkthrough

Profile a real build on Linux, scoped to the child and everything it forks:

```bash
kernelkite --label before --interval-ms 50 run before.bundle.json -- cargo build
```

Make your change, then capture the after run:

```bash
kernelkite --label after --interval-ms 50 run after.bundle.json -- cargo build
```

Print what the sampler saw:

```bash
kernelkite summary before.bundle.json
```

```text
Profile summary: before [proc]
  duration      : 1500 ms
  frames        : 7
  distinct pids : 4
  cpu seconds   : 2.790
  peak rss      : 943.00 MiB
  disk read     : 3.34 MiB
  disk write    : 2.57 MiB
  network       : (not sampled)
```

That block is the exact output from `fixtures/cargo-build-before.json`. Reproduce it with `make demo`.
