# E14: Operational Reliability

**Goal:** Make refresh, scan, cleanup, backup, restore, and scheduler workflows
recover cleanly from interruption, stale state, browser locks, and partial
failures.

## Stories

### E14-S01 Make long-running operations resumable
**Status:** todo

**Acceptance criteria**
- Refresh, scan, cleanup, backup, restore, and scheduled jobs persist enough state to explain whether the last run completed, failed, or was interrupted.
- Restarting the desktop app surfaces interrupted work without silently retrying destructive operations.
- Users can dismiss or retry recoverable interrupted work from the UI.

### E14-S02 Add idempotency to cleanup and restore execution
**Status:** todo

**Acceptance criteria**
- Re-running the same cleanup or restore request does not duplicate backup records or corrupt profile data.
- Already-completed actions are reported as completed or skipped with a clear reason.
- Tests cover repeated execution after success, partial failure, and app restart.

### E14-S03 Harden browser-lock and stale-profile handling
**Status:** todo

**Acceptance criteria**
- Locked browser artifacts are detected consistently before mutation.
- Stale profile paths, deleted profiles, and moved profiles are reported without panics or broad cleanup.
- Retry-after-close flows re-check locks before continuing.

### E14-S04 Guard concurrent manual and scheduled work
**Status:** todo

**Acceptance criteria**
- Manual scans, scheduled scans, refreshes, cleanup previews, cleanup execution, and restore execution cannot race each other into inconsistent state.
- The UI shows which operation is blocking another operation.
- Tests cover concurrent manual scan plus scheduled scan, refresh plus activation, and cleanup plus restore.

### E14-S05 Remove scheduler timing flakiness
**Status:** todo

**Acceptance criteria**
- Scheduler calculations use injectable clocks in tests.
- Missed runs, daylight-saving changes, and clock jumps produce deterministic next-run behavior.
- Tests do not compare exact timestamps from separate system-clock reads.

### E14-S06 Validate crash-recovery paths on disposable profiles
**Status:** todo

**Acceptance criteria**
- Manual validation covers interruption during scan, cleanup, backup, restore, refresh, and scheduled jobs.
- Disposable profile data remains recoverable after interrupted cleanup.
- A validation note documents observed recovery behavior and any remaining gaps.
