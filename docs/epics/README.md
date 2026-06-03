# Product Backlog

E01-E08 cover the initial technical-proof milestone. E09-E13 harden tracker
cleanup for daily-use beta. E14-E18 track maturity hardening needed before the
app should be treated like a production-grade privacy-security product. Stories
are ordered by dependency inside each epic. Status values are `todo`,
`in-progress`, and `done`.

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
| [E09](E09-cleanup-trust-boundary.md) | Make the backend authoritative for cleanup safety | 5 |
| [E10](E10-desktop-lock-workflow.md) | Integrate browser-lock handling into desktop cleanup | 5 |
| [E11](E11-granular-tracker-cleanup.md) | Remove tracker-owned data without broad profile deletion | 6 |
| [E12](E12-cleanup-recovery-and-ux.md) | Verify, explain, back up, and restore cleanup changes | 6 |
| [E13](E13-scheduled-maintenance.md) | Refresh rules and rescan safely on a schedule | 6 |
| [E14](E14-operational-reliability.md) | Recover cleanly from interruption, stale state, locks, and partial failures | 6 |
| [E15](E15-cleanup-safety-hardening.md) | Make destructive changes backend-authorized, targeted, and reversible | 6 |
| [E16](E16-browser-coverage-and-rule-quality.md) | Broaden browser coverage and improve tracker classification quality | 6 |
| [E17](E17-diagnostics-and-observability.md) | Add local privacy-preserving diagnostics and visible run states | 6 |
| [E18](E18-release-trust-and-maintenance.md) | Make builds, rule updates, rollback, and releases traceable | 6 |

## Cleanup-Hardening Traceability

| Improvement | Epic |
|-------------|------|
| 1. Generate and validate cleanup plans in Rust | [E09](E09-cleanup-trust-boundary.md) |
| 2. Integrate browser-lock handling | [E10](E10-desktop-lock-workflow.md) |
| 3. Make finding IDs profile-specific | [E09](E09-cleanup-trust-boundary.md) |
| 4. Scan and clean storage per website | [E11](E11-granular-tracker-cleanup.md) |
| 5. Improve Balanced-mode classification | [E11](E11-granular-tracker-cleanup.md) |
| 6. Add post-cleanup verification | [E12](E12-cleanup-recovery-and-ux.md) |
| 7. Add restore protection | [E12](E12-cleanup-recovery-and-ux.md) |
| 8. Explain cleanup artifact impact | [E12](E12-cleanup-recovery-and-ux.md) |
| 9. Separate tracker cleanup from general browser cleanup | [E11](E11-granular-tracker-cleanup.md) |
| 10. Schedule rescans and EasyPrivacy refreshes | [E13](E13-scheduled-maintenance.md) |

## Maturity-Hardening Traceability

| Hardening Area | Epic |
|----------------|------|
| Reliability: idempotent operations, crash recovery, stale state, browser locks, and concurrent work | [E14](E14-operational-reliability.md) |
| Safety: backend-issued previews, validated cleanup targets, rollback coverage, restore validation, and artifact policies | [E15](E15-cleanup-safety-hardening.md) |
| Coverage: more browsers, normalized profile layouts, broader artifact inventory, and stronger rule quality gates | [E16](E16-browser-coverage-and-rule-quality.md) |
| Observability: structured local run history, visible blocked/partial/failed states, diagnostic export, and stall detection | [E17](E17-diagnostics-and-observability.md) |
| Trust and maintenance: signed rule activation, bundle rollback, version traceability, reproducible releases, and update policy | [E18](E18-release-trust-and-maintenance.md) |

## Definition Of Done

- Acceptance criteria are covered by automated tests where practical.
- Privacy-sensitive logs contain no cookie values, tokens, or raw browsing
  history.
- User-facing deletion paths require a preview and explicit confirmation.
- Documentation is updated when behavior or scope changes.
