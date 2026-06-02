# E13: Scheduled Maintenance

**Goal:** Refresh tracker intelligence and rescan profiles on a user-controlled
schedule without unexpected browser disruption.

## Stories

### E13-S01 Define local scheduling preferences
**Status:** todo

**Acceptance criteria**
- Users can enable or disable scheduled rule refreshes and rescans independently.
- The UI exposes frequency, last run, next run, and last result.
- Scheduling defaults are conservative and clearly documented.

### E13-S02 Stage and validate EasyPrivacy refreshes
**Status:** todo

**Acceptance criteria**
- Refresh downloads the official EasyPrivacy subscription and compiles the supported subset.
- Attribution, selected license metadata, provenance, rule counts, and shard limits are validated before activation.
- Failed refreshes preserve the last known-good rule bundle.

### E13-S03 Activate verified rule bundles safely
**Status:** todo

**Acceptance criteria**
- New rule bundles are activated only after validation succeeds.
- Desktop and extension rule versions remain traceable to the same refresh.
- Rollback restores the previous verified bundle.

### E13-S04 Run scheduled read-only rescans
**Status:** todo

**Acceptance criteria**
- Scheduled scans remain read-only and do not close browsers.
- Results identify browser, profile, scan time, warnings, and tracker counts.
- Users can start a fresh scan manually regardless of schedule.

### E13-S05 Offer deferred cleanup after browser closure
**Status:** todo

**Acceptance criteria**
- Scheduled activity never deletes browser data automatically by default.
- Users can review findings and choose cleanup after affected browsers close.
- Notifications explain whether action is optional, blocked, or ready.

### E13-S06 Validate scheduler reliability and privacy
**Status:** todo

**Acceptance criteria**
- Tests cover missed runs, offline refreshes, invalid lists, rollback, disabled schedules, and concurrent manual scans.
- Scheduler logs contain no sensitive browsing values.
- Repeated failures produce a visible diagnostic state without retry loops.
