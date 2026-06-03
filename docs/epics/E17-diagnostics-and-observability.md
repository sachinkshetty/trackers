# E17: Diagnostics And Observability

**Goal:** Make refresh, scan, cleanup, restore, and scheduler behavior
debuggable through local, privacy-preserving diagnostics and visible run states.

## Stories

### E17-S01 Add structured local run history
**Status:** todo

**Acceptance criteria**
- Refresh, scan, cleanup, restore, and scheduled jobs write structured local run records.
- Records include run ID, start time, end time, status, browser/profile identifiers, counts, and sanitized warnings.
- Records never include cookie values, tokens, raw URLs, or raw browsing history.

### E17-S02 Surface blocked, partial, failed, and stale states in the UI
**Status:** todo

**Acceptance criteria**
- The UI distinguishes in-progress, blocked, partial success, failed, cancelled, stale, and completed states.
- Long-running operations show active progress or recent heartbeat age.
- Users can see the next recommended action for blocked or failed runs.

### E17-S03 Add privacy-preserving diagnostic export
**Status:** todo

**Acceptance criteria**
- Users can export a local diagnostics package for troubleshooting.
- Exported diagnostics redact private paths where practical and omit browsing content.
- The UI previews what categories of information will be included before export.

### E17-S04 Detect repeated failures without retry loops
**Status:** todo

**Acceptance criteria**
- Repeated refresh, scan, cleanup, restore, and scheduler failures produce a visible degraded state.
- Automatic retries use bounded backoff and stop before creating loops.
- Users can reset the failure state after reading the diagnostic reason.

### E17-S05 Add operation heartbeats and stall detection
**Status:** todo

**Acceptance criteria**
- Long-running backend operations update a local heartbeat while active.
- The UI reports when an operation appears stalled.
- Tests cover heartbeat updates, stale heartbeat detection, and cancellation.

### E17-S06 Validate diagnostic privacy
**Status:** todo

**Acceptance criteria**
- Tests scan diagnostic records and exports for cookie values, tokens, raw URLs, and raw browsing history.
- Manual review confirms diagnostics are useful enough to debug common failures.
- Privacy rules for diagnostics are documented in the backlog and user-facing copy.
