# E01: Rule Foundation

**Goal:** Represent, compile, and classify tracker rules locally and
deterministically.

## Stories

### E01-S01 Define the rule-bundle schema
**Status:** done

Create versioned Rust types for rules, categories, source metadata, and bundle
metadata.

**Acceptance criteria**
- Schema supports domain rules, categories, confidence, provenance, and license metadata.
- JSON serialization round-trips in unit tests.
- Unsupported schema versions fail clearly.

### E01-S02 Add reviewed supplemental-rule input
**Status:** todo

Define the human-editable format for product-specific rules.

**Acceptance criteria**
- Each source requires attribution and license fields.
- Invalid domains and missing metadata are rejected.

### E01-S03 Compile deterministic bundles
**Status:** todo

Build normalized desktop and Manifest V3 rule artifacts.

**Acceptance criteria**
- Identical inputs produce byte-identical output.
- Duplicate rules are resolved predictably.
- Extension rule IDs are stable.

### E01-S04 Implement local classification
**Status:** todo

Match scanned domains against verified rules and return evidence.

**Acceptance criteria**
- Exact and subdomain matching behavior is tested.
- Classification includes category, confidence, and matched rule IDs.

### E01-S05 Perform upstream list license review
**Status:** todo

Select initial public filter-list inputs only after documenting compatibility.

**Acceptance criteria**
- Each selected list has recorded source URL, license, attribution requirement, and update method.
- Incompatible sources are documented but not ingested.
