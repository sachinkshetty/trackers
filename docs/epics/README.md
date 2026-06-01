# Technical-Proof Backlog

This backlog covers the first 4-6 week milestone. Stories are ordered by
dependency inside each epic. Status values are `todo`, `in-progress`, and
`done`.

| Epic | Goal | Stories |
|------|------|---------|
| [E01](E01-rule-foundation.md) | Define deterministic local tracker rules | 5 |
| [E02](E02-profile-discovery.md) | Discover Chrome and Edge profiles safely | 4 |
| [E03](E03-read-only-scan.md) | Inspect browser data without modifying it | 6 |
| [E04](E04-cleanup-engine.md) | Preview and execute conservative cleanup | 6 |
| [E05](E05-extension-protection.md) | Block recognized trackers live | 6 |
| [E06](E06-desktop-app.md) | Expose scanning and cleanup in a Tauri UI | 5 |
| [E07](E07-signed-updates.md) | Activate only verified rule updates | 5 |
| [E08](E08-validation.md) | Prove behavior on disposable profiles | 5 |

## Definition Of Done

- Acceptance criteria are covered by automated tests where practical.
- Privacy-sensitive logs contain no cookie values, tokens, or raw browsing
  history.
- User-facing deletion paths require a preview and explicit confirmation.
- Documentation is updated when behavior or scope changes.

