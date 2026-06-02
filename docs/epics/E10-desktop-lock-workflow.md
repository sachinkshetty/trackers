# E10: Desktop Browser-Lock Workflow

**Goal:** Detect open-browser conflicts before cleanup and guide the user
through explicit, recoverable choices.

## Stories

### E10-S01 Implement a Windows resource-lock probe
**Status:** todo

**Acceptance criteria**
- Cleanup preflight identifies actions blocked by open Chrome or Edge resources.
- Lock detection is scoped to planned targets and does not mutate files.
- Tests cover locked and unlocked disposable files.

### E10-S02 Integrate preflight into desktop cleanup execution
**Status:** todo

**Acceptance criteria**
- Desktop execution always runs preflight before mutation.
- Locked actions are never attempted until a resolution is selected.
- Preflight returns locked action IDs and affected browser profiles.

### E10-S03 Present manual retry and skip choices
**Status:** todo

**Acceptance criteria**
- The UI explains when Chrome or Edge must be closed.
- The user can close browsers manually and retry, or skip locked actions.
- Cleanup results clearly distinguish completed, skipped, and failed actions.

### E10-S04 Add confirmed automatic browser closure
**Status:** todo

**Acceptance criteria**
- Automatic browser closure is offered only after explicit confirmation.
- Only the affected Chrome or Edge process family is targeted.
- The app never closes browsers implicitly during scan, preview, or cleanup.

### E10-S05 Validate lock workflows end to end
**Status:** todo

**Acceptance criteria**
- Integration tests cover unlocked execution, retry, skip, confirmed closure, and closure failure.
- A skipped locked action remains present in post-cleanup verification.
- User-facing messages identify the required next action.
