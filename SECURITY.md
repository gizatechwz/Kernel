# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| latest release on `main` | yes |
| older tags | best effort |

## Reporting a vulnerability

KernelKite reads `/proc` and writes bundles locally; it performs no
network I/O and executes no downloaded code. The realistic surface is
malformed fixtures (replay path) and malformed `/proc` snapshots (capture
path).

**How to report:** email security concerns to the maintainer rather than
opening a public issue. Include a minimal crashing fixture and the version
tag you tested against.

**What to expect:**

- acknowledgement within 7 days;
- a fix on `main` and a patch release within 30 days for confirmed
  vulnerabilities;
- credit in CHANGELOG.md and the release notes unless you prefer to stay
  anonymous.

## Scope notes

- The viewer renders bundle JSON produced by the core; it executes
  nothing from the captured data itself.
- Bundles written with `--audit`/`--out` use the caller's filesystem
  permissions; run the capture under a dedicated user on shared hosts.