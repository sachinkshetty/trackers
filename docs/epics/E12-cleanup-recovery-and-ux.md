# E12: Cleanup Recovery And UX

**Goal:** Explain cleanup impact, verify outcomes, and provide a practical
recovery path for destructive changes.

## Stories

### E12-S01 Explain artifact impact before cleanup
**Status:** done

**Acceptance criteria**
- Preview explains the likely impact of Cookies, Cache, Local Storage, IndexedDB, Service Workers, and History.
- Warnings distinguish sign-out risk, preference loss, offline-data loss, notification impact, and slower reloads.
- Tracker cleanup and general privacy cleanup use distinct labels.

### E12-S02 Add cleanup audit records
**Status:** done

**Acceptance criteria**
- Each cleanup run records timestamp, browser, profile, mode, rule-bundle version, and action outcome.
- Audit records exclude cookie values, tokens, raw history, and stored page content.
- Users can view and clear local cleanup history.

### E12-S03 Create local backups before destructive cleanup
**Status:** done

**Acceptance criteria**
- The app snapshots files or records needed to restore supported actions before mutation.
- Backup storage is local, bounded, timestamped, and excluded from telemetry.
- Backup failure blocks destructive cleanup unless the user explicitly chooses a documented no-backup path.

### E12-S04 Add restore and undo workflow
**Status:** todo

**Acceptance criteria**
- Users can restore the most recent supported cleanup within a documented retention window.
- Restore previews affected profiles and artifact types before mutation.
- Partial restore results distinguish completed, skipped, and failed actions.

### E12-S05 Verify cleanup with an automatic rescan
**Status:** todo

**Acceptance criteria**
- Cleanup triggers a lightweight rescan of affected profiles.
- Results report removed, skipped, still-detected, and failed items.
- The UI never reports full success while selected tracker artifacts remain detected.

### E12-S06 Validate backup, restore, and user messaging
**Status:** todo

**Acceptance criteria**
- Disposable-profile tests cover successful restore, partial restore, expired backups, and backup failure.
- Manual validation confirms preview warnings are understandable before deletion.
- Restore operations preserve unrelated profile data.
