# E18: Release Trust And Maintenance

**Goal:** Make builds, rule updates, rollback, and releases traceable enough for
a mature privacy-security product.

## Stories

### E18-S01 Sign and verify activated rule bundles
**Status:** todo

**Acceptance criteria**
- Rule bundles are activated only when a trusted signature and hash verify.
- Signing key IDs, bundle hashes, source metadata, and minimum client versions are recorded.
- Invalid signatures, unknown keys, and hash mismatches preserve the active bundle.

### E18-S02 Add rule bundle rollback management
**Status:** todo

**Acceptance criteria**
- Users can view the active bundle, previous verified bundle, and staged bundle.
- Rollback restores the previous verified bundle without requiring a new download.
- Rollback events are recorded in local diagnostics.

### E18-S03 Make desktop and extension rule versions traceable
**Status:** todo

**Acceptance criteria**
- Desktop scanner and browser extension can report the same refresh ID when they use the same source data.
- Generated extension shards include provenance and source version metadata.
- Mismatched versions are visible in diagnostics and settings.

### E18-S04 Add reproducible release metadata
**Status:** todo

**Acceptance criteria**
- Releases record commit SHA, app version, rule bundle version, build profile, and dependency lockfile hash.
- Release artifacts can be traced back to a GitHub tag and changelog entry.
- Build metadata is visible in the desktop settings view.

### E18-S05 Harden updater and dependency policy
**Status:** todo

**Acceptance criteria**
- Dependency updates are reviewed with changelog, license, and security-impact notes.
- The app distinguishes app updates from rule updates.
- Failed update checks do not block local scanning or cleanup.

### E18-S06 Validate release and rollback drills
**Status:** todo

**Acceptance criteria**
- A release checklist covers build, signing, rule activation, rollback, installer smoke test, and disposable-profile validation.
- Rollback drills prove app rollback and rule rollback independently.
- Results are documented without exposing local profile data.
