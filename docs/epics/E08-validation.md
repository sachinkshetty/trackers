# E08: Technical-Proof Validation

**Goal:** Demonstrate safe behavior before daily-use or private-beta distribution.

## Stories

### E08-S01 Create disposable browser-profile fixtures
**Status:** done

**Acceptance criteria**
- Fixtures include multiple profiles, seeded cookies, storage artifacts, extensions, and malformed inputs.
- Tests never target personal browser profiles.

### E08-S02 Validate cleanup preservation rules
**Status:** done

**Acceptance criteria**
- Balanced cleanup preserves ambiguous and allowlisted artifacts.
- Aggressive cleanup requires explicit confirmation.

### E08-S03 Validate locked-file workflows
**Status:** done

**Acceptance criteria**
- Retry, confirmed automatic close, and skip paths are tested.
- No browser process is closed implicitly.

### E08-S04 Validate extension behavior
**Status:** done

**Acceptance criteria**
- Standard and Strict modes, counts, pauses, and exceptions work in Chrome and Edge.
- Expected site-breakage recovery path is documented.

### E08-S05 Write milestone report
**Status:** todo

**Acceptance criteria**
- Report maps results to prototype success metrics.
- Known limitations and deferred work are documented before daily-use beta.
