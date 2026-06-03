# E15: Cleanup Safety Hardening

**Goal:** Ensure every destructive browser-data change is backend-authorized,
precisely targeted, reversible, and blocked when safety cannot be proven.

## Stories

### E15-S01 Enforce backend-issued mutation tokens
**Status:** todo

**Acceptance criteria**
- Cleanup and restore execution require backend-issued tokens tied to the latest validated preview.
- Tokens expire when scan results, rule bundles, profile discovery, or cleanup settings change.
- Tampered, stale, or replayed tokens are rejected with clear UI feedback.

### E15-S02 Validate all cleanup targets before mutation
**Status:** todo

**Acceptance criteria**
- Every cleanup target is canonicalized and verified to stay inside the intended browser profile.
- Symlinks, junctions, parent-path traversal, and missing files are handled safely.
- Tests cover path traversal, moved profile roots, and cross-profile target attempts.

### E15-S03 Require rollback coverage for destructive actions
**Status:** todo

**Acceptance criteria**
- Each destructive action declares whether it has a backup strategy, restore strategy, or explicit no-rollback reason.
- Cleanup execution blocks rollback-required actions when backup creation fails.
- The UI distinguishes reversible cleanup from irreversible cleanup before execution.

### E15-S04 Strengthen restore input validation
**Status:** todo

**Acceptance criteria**
- Restore records are validated against the original profile, artifact type, and backup manifest before writing.
- Corrupted, missing, expired, or mismatched backup records are skipped with reasons.
- Partial restore results are visible and do not hide failed records.

### E15-S05 Add artifact-specific mutation policies
**Status:** todo

**Acceptance criteria**
- Cookies, Local Storage, IndexedDB, Cache, Service Worker, History, settings, and extensions each have explicit cleanup policy metadata.
- Unsafe artifacts remain review-only until a safe mutation strategy exists.
- Balanced and aggressive modes cannot override a policy that marks an artifact unsafe.

### E15-S06 Validate cleanup isolation on disposable profiles
**Status:** todo

**Acceptance criteria**
- Tests or manual validation prove cleanup does not remove unrelated cookies, storage, settings, extensions, or profile files.
- Browser-specific cleanup for Chrome cannot affect Edge, and Edge cleanup cannot affect Chrome.
- Evidence is recorded in validation docs without including private browsing values.
