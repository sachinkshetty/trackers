# Chrome And Edge Package Validation

## Automated Validation

Run:

```powershell
cd apps/extension
npm run validate:packages
```

The command builds TypeScript, assembles unpacked `chrome` and `edge` package
directories, and verifies:

- Manifest V3 format.
- Only `declarativeNetRequest` and `storage` permissions are requested.
- Packaged starter rules exist locally.
- The compiled service worker exists.

## Browser Differences

Chrome and Edge currently use the same Chromium Manifest V3 package contents.
Keep separate output folders so browser-specific manifest overrides can be added
later without changing source modules.

## Manual Sideload Procedure

Automated checks do not replace interactive browser validation.

### Chrome

1. Open `chrome://extensions`.
2. Enable developer mode.
3. Select **Load unpacked** and choose `apps/extension/packages/chrome`.
4. Confirm the extension loads without errors.

### Edge

1. Open `edge://extensions`.
2. Enable developer mode.
3. Select **Load unpacked** and choose `apps/extension/packages/edge`.
4. Confirm the extension loads without errors.

### Controlled-Page Check

For each browser:

1. Open a local fixture page that requests one known tracker domain and one
   unrelated domain.
2. Confirm Standard mode blocks only the known tracker request.
3. Enable Strict mode and confirm the breakage warning is visible.
4. Add a temporary site exception and confirm protection pauses for that site.
5. Confirm blocked counters remain local and group results by category.

