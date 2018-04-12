# Contributing to KernelKite

KernelKite is deliberately narrow: sample a process subtree from /proc,
emit deterministic bundles, keep every number traceable to a recorded
field. Contributions that respect that scope are welcome.

## Development setup

```bash
git clone https://github.com/Mujung/KernelKite.git
cd KernelKite
cargo build
cargo test
```

The TypeScript viewer lives in `viewer/` (`npm ci && npm test`). CI
regenerates `samples/` from the shipped fixtures and fails on byte drift.

## Ground rules

- **Deterministic output.** A fixture must always replay to the same
  bytes, on any OS, forever. No wall-clock in captures, no iteration-order
  dependence in analysis, no float formatting without fixed precision.
- **Traceability.** Every derived metric must name the recorded fields it
  came from. If a number cannot be explained from the bundle, it does not
  ship.
- **Linux-first, portable everywhere.** Capture paths may be
  Linux-specific (`proc_linux.rs`, gated `ebpf.rs`); replay, compare, and
  the viewer must build on all platforms.
- **Tests on behaviour changes.** Sampler, bundle, and compare changes
  need coverage at the boundary cases (pid reuse, short intervals, empty
  subtrees).

## Commit style

Short imperative subjects (`feat: ...`, `fix: ...`, `docs: ...`). Body
only when the "why" is not obvious from the diff.

## Reporting issues

Include the command profiled, the interval, and (if possible) a fixture
bundle. Sanitize paths and environment values before attaching anything.