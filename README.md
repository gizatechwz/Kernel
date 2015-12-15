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

