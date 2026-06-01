# E03: Read-Only Browser Scan

**Goal:** Inspect supported browser artifacts and settings without changing user data.

## Stories

### E03-S01 Define finding and scan-result models
**Status:** done

**Acceptance criteria**
- Findings include profile, artifact type, site, evidence summary, confidence, and cleanup impact.
- Partial failures are represented explicitly.

### E03-S02 Scan cookies from copied databases
**Status:** todo

**Acceptance criteria**
- Scanner uses a safe read strategy for SQLite data.
- Cookie values are never logged or returned in findings.
- Locked and malformed databases produce scoped warnings.

### E03-S03 Inventory supported site storage
**Status:** todo

**Acceptance criteria**
- Local storage, IndexedDB, cache, history, and service-worker presence are reported where supported.
- Ambiguous data remains distinguishable from high-confidence tracker findings.

### E03-S04 Inventory installed extensions
**Status:** todo

**Acceptance criteria**
- Report extension identifier, display name where available, enabled state, and evidence source.
- Do not label extensions as malware.

### E03-S05 Inspect privacy-relevant settings
**Status:** todo

**Acceptance criteria**
- Report notification and sensitive site permissions, default search, homepage, proxy, and secure DNS where accessible.
- Unsupported settings are reported as unsupported, not silently omitted.

### E03-S06 Add expert-view evidence fields
**Status:** todo

**Acceptance criteria**
- Raw evidence excludes cookie values, tokens, and sensitive URL parameters.
- Redaction behavior is unit-tested.
