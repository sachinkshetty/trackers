# E05: Chromium Extension Protection

**Goal:** Block recognized tracker requests locally in Chrome and Edge.

## Stories

### E05-S01 Scaffold the Manifest V3 extension
**Status:** done

**Acceptance criteria**
- Shared extension package has `declarativeNetRequest` and `storage` permissions only.
- Starter rules load from a packaged local artifact.

### E05-S02 Build Standard-mode blocking rules
**Status:** done

**Acceptance criteria**
- Known tracker rules compile to valid Manifest V3 rules.
- Controlled fixture pages demonstrate expected blocking.

### E05-S03 Add Strict mode
**Status:** done

**Acceptance criteria**
- Strict mode is opt-in and displays a breakage warning.
- Strict-only rules are separately testable and reversible.

### E05-S04 Add per-site exceptions
**Status:** todo

**Acceptance criteria**
- User can pause protection temporarily or permanently for the current site.
- Exception state remains local.

### E05-S05 Show current-site blocked counts
**Status:** todo

**Acceptance criteria**
- Popup shows counts grouped by category.
- Counts do not leave the browser unless diagnostics are manually submitted.

### E05-S06 Validate Chrome and Edge packages
**Status:** todo

**Acceptance criteria**
- Sideloaded builds pass controlled-page tests in current Chrome and Edge.
- Browser-specific packaging differences are documented.
