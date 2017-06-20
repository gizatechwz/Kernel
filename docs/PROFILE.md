# PROFILE.md — how kernelkite measures your dev loop

This document is the reference for **what kernelkite samples, how it computes
the numbers you see, and what it deliberately does not do.** If a number shows
up in a summary or a comparison, its provenance is described here.

## Backends at a glance

| Backend   | Status                | Platform   | What it reads / does                          |
|-----------|-----------------------|------------|-----------------------------------------------|
| `proc`    | **implemented**       | Linux only | Reads `/proc` on each sampling tick.          |
| `fixture` | **implemented**       | any OS     | Replays a recorded JSON timeline verbatim.    |
| `ebpf`    | **not implemented**   | none       | Reserved for a future release. Never fakes data. |

> **On eBPF:** kernelkite ships an `EbpfSampler` type and an `ebpf` cargo
> feature, but the sampler performs no kernel work and returns
> `Error::Unsupported` on every call. There is no BPF program, no ring buffer,
> and no synthesized "eBPF" output anywhere in the codebase. The type exists so
> that the backend enum, the roadmap, and the CLI wiring are honest about the
