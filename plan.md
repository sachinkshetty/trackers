# Browser Tracker Cleaner: Brainstorm Plan

**Date:** 2026-06-01
**Status:** Interviewed product concept

## Product Thesis

Build a Windows privacy utility and companion Chromium browser extension that help users understand, remove, and block common online tracking mechanisms without claiming to be an antivirus product.

The first release targets Chrome and Microsoft Edge on Windows 10 and 11. It focuses on browser-resident data and privacy-relevant settings:

- Scan browser profiles for cookies, site storage, caches, history, service workers, installed extensions, and risky permissions.
- Classify likely trackers using licensed public rules plus a reviewed supplemental ruleset.
- Offer review, balanced, and aggressive cleanup modes.
- Block known trackers during browsing with Standard and Strict modes.
- Explain cleanup impact before deletion and support per-site exceptions.

## Research Summary

### What Can Be Cleaned

Browser cookies and local site data are stored on the machine and can be removed. Chrome documents cookies, cache, local storage, IndexedDB, and other site data as removable browsing data. Clearing cookies can sign users out and remove preferences, so cleanup must explain impact before deletion.

### What Can Be Blocked

A Chromium Manifest V3 extension can use `declarativeNetRequest` rules to block recognized tracker requests. Edge documents that Chromium extensions usually work across Chromium browsers, although API parity must still be tested. The extension should use one shared TypeScript codebase with browser-specific packaging where necessary.

### What Cannot Be Promised

- Browser fingerprinting can be reduced or flagged, but it is not a file that can be deleted.
- First-party tracking, account-based tracking, ISP visibility, and tracking performed by installed monitoring software are not solved by cookie cleanup.
- Spyware and malware detection require a separate threat-detection product design.

## Personas

### Privacy-Conscious Consumer

Wants a simple answer to: "Who may be tracking me, and what can I safely remove?"

### Advanced User

Wants evidence, raw domains, matched rules, browser profile paths, and control over exceptions.

## User Journeys

### Desktop Scan And Cleanup

1. User starts a scan.
2. App discovers Chrome and Edge profiles.
3. Scanner inspects supported browser data and privacy-relevant settings.
4. App groups findings by browser, profile, site, tracker category, and confidence.
5. User selects `Review`, `Balanced`, or `Aggressive`.
6. App previews cleanup impact.
7. If locked browser files prevent safe deletion, the user can close browsers manually, approve automatic closure, or skip locked items.
8. App performs cleanup and reports completed, skipped, and failed actions.

### Live Browser Protection

1. User installs the Manifest V3 extension.
2. Extension loads a verified local rules bundle.
3. User selects `Standard` or `Strict`.
4. Extension blocks matching tracker requests and shows per-site counts.
5. If a site breaks, the user can allowlist that site temporarily or permanently.

## MVP Decisions

| Area | Decision |
|------|----------|
| Desktop platform | Windows 10 and 11 |
| Desktop stack | Rust core with Tauri UI |
| Browsers | Chrome and Edge |
| Extension | TypeScript, Chromium Manifest V3 |
| Cleanup modes | Review, Balanced default, Aggressive |
| Blocking levels | Standard and Strict with per-site exceptions |
| Scanner coverage | Browser data plus risky browser and network settings |
| Classification | Licensed public lists plus reviewed supplemental rules |
| Updates | Signed static HTTPS bundles with embedded starter rules and last-known-good rollback |
| Data handling | Local-only by default; separate opt-ins for aggregate metrics and manually reviewed diagnostics |
| Distribution | Local prototype first, designed for later open-source release |
| First milestone | Technical proof in 4-6 weeks |

## MVP Scope

### In Scope

- Discover Chrome and Edge installations and profiles.
- Scan cookies and supported browser storage.
- Inventory browser extensions.
- Inspect notification permissions, site permissions, default search, homepage, proxy, and secure DNS settings where accessible.
- Classify findings with local rules and show confidence.
- Preview and execute cleanup actions conservatively.
- Block known trackers live in Standard mode.
- Add cautious heuristic flags in Strict mode.
- Support per-site exceptions.
- Fetch and verify signed rule bundles.
- Keep an embedded bundle and last-known-good bundle for offline use.

### Out Of Scope

- Antivirus, spyware removal, and behavioral malware detection.
- VPN, DNS filtering service, or system-wide traffic interception.
- Firefox, macOS, Linux, and mobile support.
- Cloud browsing-history analysis.
- Automatic deletion without a user-visible plan.
- Guaranteed prevention of fingerprinting or account-based tracking.

## Staged Delivery

### Milestone 1: Technical Proof

- Scan and clean disposable Chrome and Edge test profiles.
- Verify locked-file behavior and cleanup previews.
- Block known trackers on controlled test pages.
- Verify exceptions and signed rule updates.

### Milestone 2: Personal Daily-Use Beta

- Add polished Tauri UI, installer, error reporting, rollback, and regression coverage.

### Milestone 3: Small Private Beta

- Package releases for 5-20 testers.
- Add opt-in diagnostics, update rollback monitoring, and structured feedback.

## Primary Risks

| Risk | Mitigation |
|------|------------|
| Cleanup signs users out or damages profiles | Default to previewed balanced cleanup, preserve data conservatively, test disposable profiles, back up modified files where feasible |
| Browser files are locked | Scan first, then offer close, confirmed automatic closure, or skip |
| Filter-list license incompatibility | Review and record every upstream license before ingestion |
| Rule update compromise | Sign bundles, pin verification key, verify before activation, retain last-known-good bundle |
| Site breakage from blocking | Standard default, Strict opt-in, per-site exceptions, visible blocked-request details |
| Privacy product collects browsing data | Local processing by default; no URL or domain telemetry without explicit manual submission |

## Open Decisions For Implementation Planning

- Select public rules sources after license review.
- Choose the signing algorithm and key-rotation procedure.
- Decide whether desktop-extension coordination is necessary for MVP or can wait.
- Define exact heuristics allowed in Strict mode.
- Choose Windows installer and code-signing strategy for beta distribution.

## Sources

- [Chrome: delete, allow, and manage cookies](https://support.google.com/chrome/answer/95647)
- [Chrome: delete browsing data](https://support.google.com/chrome/answer/2392709)
- [Chrome Extensions: browsingData API](https://developer.chrome.com/docs/extensions/reference/api/browsingData)
- [Chrome Extensions: declarativeNetRequest API](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest)
- [Microsoft Edge: overview of Chromium extensions](https://learn.microsoft.com/en-ca/microsoft-edge/extensions-chromium/)
- [Microsoft Edge: Manifest file format](https://learn.microsoft.com/en-us/microsoft-edge/extensions/getting-started/manifest-format)
- [Microsoft Edge: native messaging](https://learn.microsoft.com/en-us/microsoft-edge/extensions-chromium/developer-guide/native-messaging)
- [Microsoft Edge: tracking prevention policy](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-browser-policies/trackingprevention)
- [Firefox: Total Cookie Protection](https://support.mozilla.org/en-US/kb/introducing-total-cookie-protection-standard-mode)

