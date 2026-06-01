# Browser Tracker Cleaner

A Windows 10 and 11 browser privacy utility with a Chrome and Edge extension.

The project detects and safely cleans browser-resident tracking data, reports
privacy-relevant settings, and blocks known tracker requests locally. It is not
an antivirus or spyware scanner.

## Workspace

| Path | Purpose |
|------|---------|
| `crates/scanner-core` | Browser discovery, scan, classification, and cleanup engine |
| `crates/scanner-cli` | Internal CLI for repeatable scanner validation |
| `crates/rule-format` | Shared rule-bundle data model |
| `crates/update-verifier` | Signed rule-bundle verification and rollback |
| `tools/rule-compiler` | Build rule bundles from reviewed inputs |
| `apps/desktop` | Tauri desktop application |
| `apps/extension` | Chromium Manifest V3 extension |
| `docs/epics` | Product backlog for the technical-proof milestone |

## Current Status

Initial scaffold only. See [docs/epics/README.md](docs/epics/README.md) for the
implementation backlog and [plan-spec.md](plan-spec.md) for the specification.
