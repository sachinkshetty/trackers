# Specification: Browser Tracker Cleaner

**Generated from:** `plan.md`  
**Interview date:** 2026-06-01  
**Version:** 1.0

## Executive Summary

Browser Tracker Cleaner is a Windows 10 and 11 desktop utility with a Chrome and Edge extension. It detects locally stored browser tracking artifacts, explains privacy-relevant settings, safely cleans selected data, and blocks known tracker requests during browsing. It is explicitly a browser privacy product, not an antivirus or spyware scanner.

## Objectives

### Primary Goals

- Give users an understandable inventory of likely browser trackers and risky settings.
- Remove tracker-related browser data without silently deleting valuable login or preference data.
- Block recognized trackers locally during browsing.
- Preserve user privacy by avoiding collection of browsing domains and URLs by default.

### Success Metrics

- Scan disposable Chrome and Edge profiles and classify seeded test artifacts.
- Execute each cleanup mode without profile corruption.
- Preserve allowlisted data and report locked or skipped items accurately.
- Block expected tracker requests on controlled pages in both browsers.
- Activate only correctly signed rule bundles and recover to the last verified bundle after an invalid update.

## Scope

### In Scope

- Chrome and Edge profile discovery on Windows 10 and 11.
- Cookie and supported site-storage inspection.
- Cache, history, service worker, and extension inventory reporting.
- Inspection of notification permissions, site permissions, default search, homepage, proxy, and secure DNS settings where accessible.
- Local classification of findings.
- Review, Balanced, and Aggressive cleanup modes.
- Standard and Strict extension blocking levels.
- Temporary and permanent per-site exceptions.
- Signed static rule updates over HTTPS.
- Optional aggregate metrics and manual diagnostics, each disabled by default.

### Out Of Scope

- Malware, spyware, keylogger, or stalkerware detection.
- VPN functionality or system-wide packet inspection.
- Guaranteed fingerprinting prevention.
- Firefox, macOS, Linux, and mobile support.
- Automatic remote collection of domains, URLs, or browsing history.

### Future Considerations

- Firefox support.
- Scheduled scans and cleanup reminders.
- Richer trend reports and exports.
- Separate Windows trace cleanup module.
- Separate threat-detection product research.

## Technical Requirements

### Architecture

Use a Rust workspace with a reusable scanner core, a CLI harness for repeatable testing, and a Tauri desktop application. Build one Manifest V3 TypeScript extension and produce Chrome and Edge packages from the same source. Use a rule compiler to normalize licensed upstream lists and reviewed product-specific rules into desktop and extension artifacts.

The scanner must plan cleanup before changing data. It must record the affected profile, artifact type, estimated impact, and whether browser closure is required. Cleanup should operate only after user confirmation.

### Components

| Component | Purpose | Technology |
|-----------|---------|------------|
| Scanner core | Profile discovery, inspection, classification, cleanup planning, cleanup execution | Rust |
| Scanner CLI | Automated and manual engine validation | Rust |
| Desktop app | Scan, review, cleanup, settings, update UI | Tauri with TypeScript UI |
| Browser extension | Live blocking, badge counts, allowlists, protection levels | TypeScript, Manifest V3 |
| Rule compiler | Normalize sources, validate licenses metadata, emit artifacts | Rust or TypeScript build tool |
| Update verifier | Download, verify, activate, and roll back rule bundles | Rust for desktop; browser-compatible verification for extension |
| Static update host | Serve version manifest and signed bundles | HTTPS object storage or static hosting |

### Suggested Repository Layout

```text
apps/
  desktop/
  extension/
crates/
  scanner-core/
  scanner-cli/
  rule-format/
  update-verifier/
tools/
  rule-compiler/
rules/
  supplemental/
docs/
```

### Extension Permissions

Use the smallest permission set that meets the release scope. Expect `declarativeNetRequest`, `storage`, and required host permissions. Add permissions such as `browsingData`, `management`, or `nativeMessaging` only when a reviewed feature requires them. Avoid remote code: rule updates are data bundles, not executable JavaScript.

### Desktop And Extension Coordination

The MVP may operate without direct coordination: the desktop app handles deep scans and cleanup, while the extension handles request blocking. If a later workflow requires coordination, use Chromium native messaging with a narrow message schema and an allowlisted extension origin.

## Data Models

### Finding

```text
Finding {
  id
  browser
  profile_id
  artifact_type
  site
  tracker_domain?
  tracker_category?
  confidence
  matched_rule_ids[]
  cleanup_impact
  requires_browser_closed
  evidence_summary
}
```

### CleanupPlan

```text
CleanupPlan {
  id
  mode
  created_at
  actions[]
  warnings[]
  estimated_counts
}

CleanupAction {
  finding_ids[]
  operation
  target
  requires_browser_closed
  reversible
}
```

### RuleBundleManifest

```text
RuleBundleManifest {
  schema_version
  bundle_version
  generated_at
  min_app_version
  artifacts[]
  sha256
  signature
  signing_key_id
  upstream_sources[]
}
```

### SiteException

```text
SiteException {
  site
  protection_override
  expires_at?
  created_by_user
}
```

## Functional Requirements

### Desktop Scanner

- Discover standard Chrome and Edge profile locations.
- Treat each profile independently.
- Show scan progress and partial failures.
- Never modify browser files during scan.
- Classify findings locally using a verified rules bundle.
- Separate likely tracker data from ambiguous site data.
- Show raw evidence only in expert view.

### Cleanup Modes

| Mode | Behavior |
|------|----------|
| Review | User selects individual findings or grouped findings |
| Balanced | Remove high-confidence tracker artifacts while preserving likely login and preference data |
| Aggressive | Remove broader browser data only after explicit sign-out and site-functionality warnings |

Before executing a cleanup plan, detect locked resources. Offer three choices: close browsers manually and retry, approve automatic browser closure, or skip locked items. Automatic closure must require explicit confirmation and warn about active downloads and unsaved forms.

### Risky Settings Inspection

- Report suspicious or unexpected extensions without automatically labeling them malware.
- Report notification permissions and sensitive site permissions.
- Report default search engine and homepage changes.
- Report proxy and secure DNS configuration where supported.
- Explain why each setting matters and link to remediation steps.

### Extension

- Use Manifest V3 `declarativeNetRequest` rules for recognized tracker blocking.
- Default to `Standard`.
- Offer `Strict` as an explicit choice with clear breakage warning.
- Show per-site blocked counts and grouped categories.
- Allow temporary and permanent exceptions.
- Provide a one-click pause for the current site.
- Store rules, exceptions, and counts locally.

### Rule Updates

- Ship an embedded starter bundle.
- Periodically fetch a version manifest over HTTPS.
- Verify cryptographic signature and content hash before activation.
- Retain the last verified bundle.
- Fall back to embedded rules if no verified external bundle exists.
- Record source attribution and license metadata in each generated bundle.

### Telemetry And Diagnostics

- Do not send browsing domains, URLs, cookie values, or history by default.
- Make aggregate metrics opt-in and separate from diagnostics.
- Permit manual diagnostic submission only after the user reviews the included fields.
- Redact cookie values, tokens, and personally identifying URL parameters from diagnostics.

## User Experience

### Main Screens

| Screen | Purpose |
|--------|---------|
| Dashboard | Start scan, see protection state, show last scan summary |
| Scan results | Summary, expandable grouped findings, cleanup mode selection |
| Expert view | Raw domains, rule IDs, artifact types, paths, and setting values |
| Cleanup preview | Actions, sign-out warnings, locked-item choices |
| Extension popup | Current-site tracker count, protection level, pause, exception |
| Settings | Updates, telemetry opt-ins, diagnostics, rule version |

### Edge Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| Browser profile is locked | Scan what is safe; require user choice before cleanup |
| Rule download fails | Continue with last verified bundle |
| Rule signature is invalid | Reject update and retain last verified bundle |
| Site breaks after blocking | Let user pause protection or add an exception |
| Ambiguous cookie classification | Exclude from Balanced cleanup and explain uncertainty |
| Multiple browser profiles exist | Scan and report each profile separately |
| Cleanup partially fails | Report completed, skipped, and failed actions without claiming full success |

## Security Requirements

- Use memory-safe Rust for the scanner core.
- Never upload raw browser databases.
- Never log cookie values, tokens, or sensitive URL parameters.
- Validate all rule-bundle inputs before parsing or activation.
- Pin trusted rule-signing public keys in released clients.
- Define key rotation before public beta.
- Use least-privilege extension permissions.
- Avoid administrative privileges unless a specific inspected Windows setting requires them.
- Build tests around hostile or malformed profile databases and rule bundles.

## Testing Strategy

### Automated Tests

- Unit-test classification, cleanup planning, redaction, signature verification, and rollback.
- Create seeded disposable Chrome and Edge fixture profiles.
- Verify Balanced cleanup preserves ambiguous and allowlisted artifacts.
- Verify Aggressive cleanup requires explicit confirmation state.
- Test malformed SQLite data, missing files, locked files, multiple profiles, and partial failures.
- Test rule compiler determinism and license metadata inclusion.
- Test Standard and Strict extension rules against controlled fixture pages.

### Manual Validation

- Sideload the extension in current Chrome and Edge releases.
- Run cleanup only against disposable profiles during Milestone 1.
- Validate active-download and unsaved-form warnings before testing confirmed browser closure.
- Review extension store permission warnings before broader distribution.

## Deployment Requirements

### Prototype

- Local Tauri builds.
- Internal CLI builds.
- Sideloaded Chrome and Edge extension packages.
- Static HTTPS rule host with test signing keys.

### Later Open-Source Release

- Publish source, build instructions, upstream rule attributions, and release checksums.
- Create signed Windows installer builds.
- Submit store packages to Chrome Web Store and Microsoft Edge Add-ons.
- Replace test signing keys with release keys and documented rotation procedure.

## Delivery Plan

### Weeks 1-2

- Scaffold workspace and rule format.
- Implement browser discovery and read-only profile scans.
- Build seeded fixture profiles and classifier tests.

### Weeks 3-4

- Implement cleanup planning and controlled cleanup execution.
- Add locked-file behavior and partial-failure reporting.
- Build extension blocking, local counts, modes, and exceptions.

### Weeks 5-6

- Add signed static rule updates and rollback.
- Build Tauri prototype UI.
- Complete Chrome and Edge technical-proof validation.

## Key Trade-Offs

- **Decision:** Preserve data conservatively in Balanced mode.  
  **Rationale:** A privacy tool loses trust if it unexpectedly signs users out or breaks profiles.

- **Decision:** Perform classification locally.  
  **Rationale:** Sending visited domains to a service creates the privacy problem the product is meant to reduce.

- **Decision:** Keep spyware detection out of MVP.  
  **Rationale:** Browser privacy cleanup and endpoint threat detection require different evidence, update pipelines, and user claims.

- **Decision:** Validate locally before public release.  
  **Rationale:** Browser-data deletion needs disposable-profile tests and careful failure handling before broad distribution.

## Deferred Decisions

- Exact licensed public rule sources.
- Signing algorithm, release key custody, and rotation process.
- Strict-mode heuristic rules.
- Desktop-extension native messaging requirement.
- Windows installer framework and binary code-signing provider.

---

*This specification was generated through a brainstorm and iterative requirements interview.*
