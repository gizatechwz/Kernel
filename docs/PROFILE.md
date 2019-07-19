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
> intended future shape without pretending the work is done. See
> `crates/kernelkite-core/src/ebpf.rs`.

## The sampling loop

kernelkite is a **sampling** profiler, not a tracer. On a fixed cadence
(`--interval-ms`, default 100 ms) it takes a snapshot of every process in scope
and records it as one *frame*. A profile bundle is an ordered list of frames
plus capture metadata.

```
t=0ms     t=100ms    t=200ms    t=300ms   ...
[frame]   [frame]    [frame]    [frame]
  |          |          |          |
  +-- per-process CPU / memory / IO / fd counts, + optional network summary
```

Because it samples rather than intercepts syscalls, kernelkite has low, bounded
overhead and needs no elevated privileges beyond read access to the target
processes' `/proc` entries. The tradeoff is temporal resolution: events shorter
than one interval may fall between frames. For a build/test loop that runs for
hundreds of milliseconds to minutes, a 50–200 ms interval captures the shape
faithfully.

## What is read on Linux (`proc` backend)

For each selected pid, per frame:

| Field           | Source                        | Notes                                                |
|-----------------|-------------------------------|------------------------------------------------------|
| `ppid`, `comm`  | `/proc/<pid>/stat`            | `comm` is parsed robustly even with spaces/parens.   |
| `utime`,`stime` | `/proc/<pid>/stat`            | CPU time in clock ticks since process start (cumulative). |
| `rss_bytes`     | `/proc/<pid>/statm` (resident)| `resident_pages × page_size`.                        |
| `vsize_bytes`   | `/proc/<pid>/statm` (size)    | `size_pages × page_size`.                            |
| `threads`       | `/proc/<pid>/stat`            | `num_threads`.                                       |
| `read_bytes`    | `/proc/<pid>/io`              | Storage-layer bytes, cumulative. `None` if unreadable. |
| `write_bytes`   | `/proc/<pid>/io`              | Storage-layer bytes, cumulative. `None` if unreadable. |
| `open_fds`      | `/proc/<pid>/fd` (dir count)  | Count of entries. `None` if unreadable.              |

Host-wide, once per frame when `--network` is set:

| Field              | Source            | Notes                                              |
|--------------------|-------------------|----------------------------------------------------|
| `network.rx_bytes` | `/proc/net/dev`   | Sum of RX bytes across non-loopback interfaces.    |
| `network.tx_bytes` | `/proc/net/dev`   | Sum of TX bytes across non-loopback interfaces.    |
| `network.interfaces` | `/proc/net/dev` | Which interfaces contributed.                      |

### Why the network summary is host-wide

`/proc/net/dev` reports counters **per interface**, not per process. Attributing
bytes to a specific process requires either connection-table correlation
(`/proc/net/tcp` + socket inodes, which is racy and coarse) or a kernel probe
(eBPF). kernelkite therefore reports network as an honest host-wide *summary*
and leaves per-process attribution to the future eBPF backend. It never guesses.

### Reading `/proc/<pid>/stat` correctly

The `comm` field is wrapped in parentheses and can itself contain spaces and
parentheses, e.g. `1234 ((my cmd)) S 1000 ...`. Splitting the whole line on
whitespace therefore breaks. kernelkite locates the **last** `)` to terminate
`comm`, then splits the remaining fields positionally per `proc(5)`. This is
unit-tested in `proc_linux.rs::tests::parse_stat_handles_spaces_and_parens_in_comm`.

## Process scope (pid filters)

The `run` command profiles the **subtree** of the command it launches: the
child pid plus every descendant observed in that frame. This is recomputed each
frame, so compilers/linkers a build tool forks are captured as they appear and
drop out as they exit. The core also supports `PidFilter::All` and an explicit
`PidFilter::Set`.

## Derived metrics

Raw bundles store cumulative kernel counters. `kernelkite summary` turns them
into the aggregates a developer cares about:

- **cpu_seconds** — for each pid, `(last_cpu_ticks − first_cpu_ticks)` summed
  across pids, divided by the host's `clock_ticks_per_sec`. This measures CPU
  *consumed during the capture window*, robust to processes that started before
  sampling began.
- **peak_rss_bytes** — the maximum, over all frames, of the summed resident
  memory of every process alive in that frame. A conservative "how much memory
  did this loop need at once" figure.
- **read_bytes / write_bytes** — the summed delta of each pid's cumulative I/O
  counters. `None` when `/proc/<pid>/io` was not readable for any process.
- **net_rx_bytes / net_tx_bytes** — last-minus-first of the host-wide network
  summary. Present only if `--network` was used.
- **duration_ms / frames / distinct_pids** — timeline shape.

### CPU% in the viewer

The viewer document computes a per-frame CPU **percentage** from the delta of
cumulative ticks between adjacent frames:

```
cpu_pct = (Δticks / clock_ticks_per_sec) / Δt_seconds × 100
```

A value above 100 is expected and correct for a multi-threaded process using
more than one core during the interval.

## Bundle format (schema v1)

A bundle is pretty-printed JSON with a stable field order, so the same in-memory
bundle always serializes to identical bytes (verified by
`bundle_json_roundtrip_is_stable`). Top-level shape:

```jsonc
{
  "meta": {
    "schema_version": 1,
    "backend": "proc" | "fixture" | "ebpf",
    "label": "before",
    "command": ["cargo", "build"],      // present for live runs
    "clock_ticks_per_sec": 100,
    "page_size_bytes": 4096,
    "interval_ms": 100,
    "started_unix_secs": 1735689600,    // omitted for deterministic fixtures
    "host_os": "linux",
    "exit_code": 0                        // present when a command was run
  },
  "frames": [
    {
      "t_ms": 0,
      "processes": [                      // serialized as an array, keyed by pid in memory
        { "pid": 5000, "ppid": 4200, "comm": "cargo",
          "utime_ticks": 8, "stime_ticks": 3,
          "rss_bytes": 20971520, "vsize_bytes": 524288000,
          "threads": 6, "read_bytes": 65536, "write_bytes": 4096, "open_fds": 18 }
      ],
      "network": { "rx_bytes": 0, "tx_bytes": 0, "interfaces": ["eth0"] } // optional
    }
  ]
}
```

Loading validates `schema_version`; a mismatch is a hard error
(`schema_mismatch_is_rejected`).

## Determinism

- **Fixtures** never record wall-clock time and replay their own recorded
  timestamps, so a replayed bundle is byte-for-byte reproducible
  (`fixture_replay_is_deterministic`).
- **BTreeMap** ordering for processes and stable struct field order make JSON
  output canonical.
- Fixture frames must be non-decreasing in `t_ms`; out-of-order input is
  rejected (`fixture_rejects_out_of_order_frames`).

This is what makes before/after comparisons trustworthy: the only thing that
changed between two fixture-based runs is the input you changed.

## Before/after comparisons
