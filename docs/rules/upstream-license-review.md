# Upstream Rule-Source License Review

**Reviewed:** 2026-06-01

## Decision

The technical-proof release compiles the supported pure domain-block subset of
EasyPrivacy and preserves upstream attribution alongside generated artifacts.

This is deliberate. Generated desktop and extension artifacts redistribute rule
data. Before adding an upstream list, the project must choose a distribution
license and attribution strategy that satisfies the upstream terms.

The machine-readable source registry is in
[`rules/upstream-sources.json`](../../rules/upstream-sources.json).

## Selected Prototype Source

| Source | License | Attribution | Update Method | Status |
|--------|---------|-------------|---------------|--------|
| [EasyPrivacy](https://easylist.to/pages/licence.html) | `CC-BY-SA-3.0-or-later` | `The EasyList authors (https://easylist.to/)` | Fetch the official subscription and compile pure domain-block rules | Selected |

## Deferred Candidates

| Source | License | Attribution | Update Method | Decision |
|--------|---------|-------------|---------------|----------|
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

## Refreshing EasyPrivacy

Run the importer from the extension workspace:

```powershell
cd apps/extension
npm run import:easyprivacy
```

The importer downloads the official subscription, compiles supported pure
domain-block rules into Chromium static-rule shards, updates the extension
manifest, and refreshes the embedded desktop bundle.

Chromium guarantees an extension at least 30,000 static rules. Any generated
rules above that threshold depend on the static-rule capacity available in the
user's browser.

