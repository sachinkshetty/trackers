# E04: Cleanup Engine

**Goal:** Build previewable, conservative, and auditable browser cleanup.

## Stories

### E04-S01 Define cleanup plan and action models
**Status:** done

**Acceptance criteria**
- Plans record mode, targets, warnings, estimated counts, and closure requirements.
- Scanning and planning do not modify profiles.

### E04-S02 Generate Review-mode plans
**Status:** done

**Acceptance criteria**
- User-selected findings map to explicit actions.
- Invalid or stale selections fail safely.

### E04-S03 Generate Balanced-mode plans
**Status:** done

**Acceptance criteria**
- Only high-confidence tracker artifacts are selected automatically.
- Ambiguous, login-related, and allowlisted data are preserved by default.

### E04-S04 Generate Aggressive-mode plans
**Status:** done

**Acceptance criteria**
- Broader deletion requires an explicit confirmation state.
- Sign-out and site-functionality warnings are always present.

### E04-S05 Handle locked browser resources
**Status:** done

**Acceptance criteria**
- Locked targets are detected before mutation.
- Caller can retry after manual close, request confirmed automatic closure, or skip locked actions.
- Active browser processes are never closed without explicit confirmation.

### E04-S06 Execute and report partial cleanup
**Status:** done

**Acceptance criteria**
- Result distinguishes completed, skipped, and failed actions.
- Failure never produces a false full-success report.
- Disposable-profile integration tests cover partial failure.
