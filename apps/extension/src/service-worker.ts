type ProtectionLevel = "standard" | "strict";

const DEFAULT_LEVEL: ProtectionLevel = "standard";
const DEFAULT_EXCEPTIONS: unknown[] = [];
const DEFAULT_BLOCKED_COUNTS = {};

chrome.runtime.onInstalled.addListener(async () => {
  const state = await chrome.storage.local.get([
    "protectionLevel",
    "siteExceptions",
    "blockedCounts",
  ]);
  if (!state.protectionLevel) {
    await chrome.storage.local.set({ protectionLevel: DEFAULT_LEVEL });
  }
  if (!state.siteExceptions) {
    await chrome.storage.local.set({ siteExceptions: DEFAULT_EXCEPTIONS });
  }
  if (!state.blockedCounts) {
    await chrome.storage.local.set({ blockedCounts: DEFAULT_BLOCKED_COUNTS });
  }
});
