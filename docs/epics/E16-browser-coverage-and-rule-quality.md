# E16: Browser Coverage And Rule Quality

**Goal:** Broaden supported browser/profile coverage and improve tracker
classification quality without overreaching into unrelated user data.

## Stories

### E16-S01 Expand browser discovery beyond Chrome and Edge
**Status:** todo

**Acceptance criteria**
- Firefox and Brave profile discovery are supported behind clearly labeled capability flags.
- Unsupported browsers are reported as unsupported instead of ignored silently.
- Discovery tests cover default, custom, missing, and locked profile roots.

### E16-S02 Normalize browser profile layouts
**Status:** todo

**Acceptance criteria**
- Profile metadata uses a common model for browser family, profile name, profile path, and artifact capabilities.
- Browser-specific paths are resolved through dedicated adapters rather than scattered conditional logic.
- Tests cover profile names with spaces, non-default profiles, and missing metadata files.

### E16-S03 Broaden artifact inventory safely
**Status:** todo

**Acceptance criteria**
- The scanner inventories additional supported browser artifacts only when they can be read safely.
- Artifacts without safe cleanup support are shown as review-only findings.
- Warnings distinguish unreadable, unsupported, locked, and malformed artifacts.

### E16-S04 Improve tracker rule source quality gates
**Status:** todo

**Acceptance criteria**
- Rule sources define license, attribution, provenance URL, supported parser, expected minimum rule count, and activation policy.
- Refresh rejects rule sets with suspiciously low counts, malformed domains, duplicate conflicts, or missing provenance.
- Rule quality failures preserve the current active bundle.

### E16-S05 Add classification confidence explanations
**Status:** todo

**Acceptance criteria**
- Findings explain why a site or artifact is considered tracker-owned, ambiguous, or general browser data.
- Confidence labels map to rule source, artifact type, and evidence quality.
- Expert view exposes classification evidence without showing cookie values or browsing content.

### E16-S06 Measure coverage on disposable browser fixtures
**Status:** todo

**Acceptance criteria**
- Disposable fixtures cover each supported browser and artifact type.
- Fixture scans report expected tracker counts and expected review-only items.
- Coverage gaps are documented as backlog items rather than hidden in test setup.
