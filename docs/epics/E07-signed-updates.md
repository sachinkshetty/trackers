# E07: Signed Rule Updates

**Goal:** Download and activate rules without trusting the network or hosting layer.

## Stories

### E07-S01 Define the update manifest
**Status:** done

**Acceptance criteria**
- Manifest includes schema version, bundle version, hash, signature, signing key ID, minimum client version, and source metadata.

### E07-S02 Verify hashes and signatures
**Status:** todo

**Acceptance criteria**
- Invalid hashes, signatures, keys, and schema versions are rejected.
- Verification has malformed-input tests.

### E07-S03 Implement last-known-good activation
**Status:** todo

**Acceptance criteria**
- New bundle activates only after verification.
- Previous verified bundle remains available after failed activation.

### E07-S04 Add embedded starter-bundle fallback
**Status:** todo

**Acceptance criteria**
- App and extension remain functional offline.
- Embedded fallback is used when no external verified bundle exists.

### E07-S05 Document key custody and rotation
**Status:** todo

**Acceptance criteria**
- Prototype test-key workflow is documented.
- Public-beta release-key storage and rotation requirements are defined before release.
