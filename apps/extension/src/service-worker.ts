type ProtectionLevel = "standard" | "strict";

const DEFAULT_LEVEL: ProtectionLevel = "standard";
const DEFAULT_EXCEPTIONS: unknown[] = [];

chrome.runtime.onInstalled.addListener(async () => {
  const state = await chrome.storage.local.get([
    "protectionLevel",
    "siteExceptions",
  ]);
  if (!state.protectionLevel) {
    await chrome.storage.local.set({ protectionLevel: DEFAULT_LEVEL });
  }
  if (!state.siteExceptions) {
    await chrome.storage.local.set({ siteExceptions: DEFAULT_EXCEPTIONS });
  }
});
