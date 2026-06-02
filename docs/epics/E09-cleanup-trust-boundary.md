# E09: Cleanup Trust Boundary

**Goal:** Make Rust the sole authority for cleanup plans and ensure every
deletion target belongs to an approved browser profile.

## Stories

### E09-S01 Generate cleanup plans only in the backend
**Status:** todo

**Acceptance criteria**
- The desktop UI sends finding selections and cleanup mode, never filesystem deletion paths.
- Rust maps scan findings to cleanup actions and returns the preview.
- Cleanup execution accepts only a backend-issued preview or backend plan identifier.

### E09-S02 Validate cleanup targets against discovered profiles
**Status:** todo

**Acceptance criteria**
- Every cleanup target is canonicalized and verified to remain inside a discovered Chrome or Edge profile.
- Traversal attempts, unapproved roots, and malformed paths fail before mutation.
- Tests prove that arbitrary user-supplied paths cannot be deleted.

### E09-S03 Make finding and action IDs profile-specific
**Status:** todo

**Acceptance criteria**
- Finding IDs include browser family, profile identity, artifact type, and site or storage key.
- The same cookie host in Chrome, Edge, and multiple profiles produces distinct IDs.
- Skipping or selecting one action cannot affect another profile's action.

### E09-S04 Remove duplicate cleanup planning from the UI
**Status:** todo

**Acceptance criteria**
- JavaScript no longer reconstructs cleanup actions or joins filesystem paths.
- Preview invokes the Tauri `preview_cleanup` command and renders its response.
- Existing Review, Balanced, Aggressive, and browser-filter controls continue to work.

### E09-S05 Add adversarial trust-boundary tests
**Status:** todo

**Acceptance criteria**
- Tests cover stale findings, duplicate IDs, traversal paths, unapproved profiles, and tampered previews.
- Failed validation leaves disposable profile fixtures unchanged.
- Logs and errors do not expose sensitive browser values.
