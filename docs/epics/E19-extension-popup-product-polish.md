# E19: Extension Popup Product Polish

**Goal:** Turn the extension popup from a development counter into a polished,
trustworthy companion UI for live tracker blocking.

## Stories

### E19-S01 Replace development counter with protection overview
**Status:** todo

**Acceptance criteria**
- The popup no longer uses development-only copy such as "Development Tracker Counter".
- The first screen clearly shows whether protection is on for the current browser session.
- Total blocked tracker requests are visible without making the popup feel crowded.
- Empty states explain that blocking is active even when no requests have been blocked yet.

### E19-S02 Show current-site blocking status
**Status:** todo

**Acceptance criteria**
- The popup identifies the current site when a tab URL is available.
- Current-site blocked counts are shown separately from all-time totals.
- Unknown, internal, extension, and restricted browser pages show a safe fallback state.
- Tests cover popup rendering with a normal site, no active tab, and unsupported URLs.

### E19-S03 Add pause and resume controls for the current site
**Status:** todo

**Acceptance criteria**
- Users can pause blocking for the current site temporarily or permanently.
- Users can resume blocking for a paused site from the popup.
- The UI clearly distinguishes temporary pause, permanent pause, and active protection.
- Existing site-exception logic is reused rather than duplicated in the popup.

### E19-S04 Present top blocked sites and categories cleanly
**Status:** todo

**Acceptance criteria**
- Top blocked sites are shown in a compact ranked list.
- Category counts such as analytics and advertising are visible where available.
- The list has clear empty, loading, and error states.
- Clearing counts remains available but is visually secondary and requires deliberate action.

### E19-S05 Surface rule and package metadata
**Status:** todo

**Acceptance criteria**
- The popup shows the active rule source or bundle version where available.
- EasyPrivacy attribution and license information are reachable without crowding the main view.
- Chrome and Edge packages expose consistent metadata.
- Package validation checks that required popup metadata is present.

### E19-S06 Align popup visual design with the desktop app
**Status:** todo

**Acceptance criteria**
- The popup uses the same calm privacy-product tone as the desktop UI.
- Layout fits common extension popup widths without clipping or horizontal scrolling.
- Buttons, counters, status badges, and warnings have consistent visual hierarchy.
- Accessibility checks cover keyboard navigation, readable contrast, and meaningful labels.

### E19-S07 Validate extension popup behavior end to end
**Status:** todo

**Acceptance criteria**
- Automated tests cover count rendering, site pause/resume, current-site fallbacks, and reset behavior.
- Manual validation covers loading the unpacked Chrome and Edge packages.
- The popup still works when storage is empty, corrupted, or partially missing.
- Validation notes confirm the popup does not expose raw browsing URLs beyond the current site hostname.
