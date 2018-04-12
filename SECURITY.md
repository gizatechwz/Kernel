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
path). Email security concerns rather than opening a public issue;
include a minimal crashing fixture and the version tag. Fixes land in the
next patch release and are credited in CHANGELOG.md.