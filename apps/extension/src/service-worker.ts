type ProtectionLevel = "standard" | "strict";

const DEFAULT_LEVEL: ProtectionLevel = "standard";

chrome.runtime.onInstalled.addListener(async () => {
  const state = await chrome.storage.local.get("protectionLevel");
  if (!state.protectionLevel) {
    await chrome.storage.local.set({ protectionLevel: DEFAULT_LEVEL });
  }
});

