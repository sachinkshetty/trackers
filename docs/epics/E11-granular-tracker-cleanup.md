# E11: Granular Tracker Cleanup

**Goal:** Identify and remove tracker-owned browser data without deleting broad
profile directories by default.

## Stories

### E11-S01 Inventory storage by origin where supported
**Status:** todo

**Acceptance criteria**
- Local Storage, IndexedDB, Cache, and Service Worker findings identify origins where Chromium storage layout permits it.
- Findings retain an ambiguous profile-level fallback when an origin cannot be determined safely.
- Scanning remains read-only and avoids exposing stored values.

### E11-S02 Classify storage origins with tracker rules
**Status:** todo

**Acceptance criteria**
- Storage origins are matched against the active tracker-rule bundle.
- Findings distinguish known tracker-owned, site-owned, and ambiguous storage.
- Classification records provenance and confidence.

### E11-S03 Add precise deletion strategies per artifact type
**Status:** todo

**Acceptance criteria**
- Known tracker-owned origins can be cleaned without deleting unrelated site data where technically supported.
- Broad profile-directory deletion remains Aggressive-only and requires a specific warning.
- Unsupported precise deletion paths are reported rather than guessed.

### E11-S04 Improve Balanced-mode cookie selection
**Status:** todo

**Acceptance criteria**
- Known tracker-only cookie domains can be selected automatically.
- Login-sensitive, allowlisted, site-owned, and ambiguous cookies are preserved by default.
- Classification fixtures cover tracker cookies, authentication cookies, and shared-domain edge cases.

### E11-S05 Separate tracker cleanup from general browser cleanup
**Status:** todo

**Acceptance criteria**
- The UI presents known tracker data separately from general cache, history, and ambiguous site data.
- The default cleanup path targets known tracker data only.
- General browser cleanup requires an explicit user selection and impact warning.

### E11-S06 Validate granular preservation rules
**Status:** todo

**Acceptance criteria**
- Disposable profiles contain multiple sites sharing each supported artifact type.
- Tests prove targeted cleanup preserves unrelated origins and allowlisted data.
- Tests document artifact types that remain Aggressive-only.
