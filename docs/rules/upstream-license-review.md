# Upstream Rule-Source License Review

**Reviewed:** 2026-06-01

## Decision

The technical-proof release compiles only manually reviewed supplemental rules
owned by this project. No third-party filter list is ingested yet.

This is deliberate. Generated desktop and extension artifacts redistribute rule
data. Before adding an upstream list, the project must choose a distribution
license and attribution strategy that satisfies the upstream terms.

The machine-readable source registry is in
[`rules/upstream-sources.json`](../../rules/upstream-sources.json).

## Selected Prototype Source

| Source | License | Attribution | Update Method | Status |
|--------|---------|-------------|---------------|--------|
| Browser Tracker Cleaner supplemental rules | `MIT OR Apache-2.0` | Browser Tracker Cleaner contributors | Reviewed pull requests to `rules/supplemental/` | Selected |

## Deferred Candidates

| Source | License | Attribution | Update Method | Decision |
|--------|---------|-------------|---------------|----------|
| [EasyPrivacy](https://easylist.to/pages/licence.html) | Dual licensed: `GPL-3.0-or-later` or `CC-BY-SA-3.0-or-later` | Attribute `The EasyList authors (https://easylist.to/)` when required | Fetch an official EasyPrivacy subscription and compile supported rules | Defer until share-alike distribution obligations and parser scope are approved |
| [AdGuard Filters](https://github.com/AdguardTeam/AdguardFilters) | `GPL-3.0` | Preserve required notices and license terms | Fetch reviewed privacy-filter inputs from the upstream repository | Defer until GPL distribution strategy and parser scope are approved |
| [DuckDuckGo Tracker Blocklists](https://github.com/duckduckgo/tracker-blocklists) | `CC-BY-NC-SA-4.0` | Preserve attribution and share-alike terms | Fetch published tracker-blocklist artifacts | Exclude from default ingestion because the non-commercial restriction conflicts with an unrestricted consumer-product path |

## Review Checklist For New Sources

Before enabling a third-party source:

1. Record its canonical URL, exact license, attribution requirements, and
   update method in `rules/upstream-sources.json`.
2. Confirm the intended distribution model is compatible with its terms.
3. Define which upstream syntax is supported and test unsupported-rule
   handling.
4. Include provenance and license metadata in generated bundles.
5. Review updates before promotion into signed production bundles.

## Notes

This file records an engineering distribution decision, not legal advice.

