# E02: Browser Profile Discovery

**Goal:** Find Chrome and Edge profiles on Windows without modifying them.

## Stories

### E02-S01 Define browser and profile models
**Status:** todo

**Acceptance criteria**
- Models identify browser family, installation, profile name, path, and discovery warnings.
- Paths remain profile-scoped.

### E02-S02 Discover standard Chrome profiles
**Status:** todo

**Acceptance criteria**
- Default and named profiles are discovered.
- Missing or inaccessible profile roots return warnings instead of crashes.

### E02-S03 Discover standard Edge profiles
**Status:** todo

**Acceptance criteria**
- Default and named profiles are discovered.
- Discovery logic is covered with fixture directories.

### E02-S04 Expose discovery through the scanner CLI
**Status:** todo

**Acceptance criteria**
- CLI prints structured JSON for automation.
- CLI performs no writes to discovered profiles.

