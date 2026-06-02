import {
  recordDebugRuleMatch,
  type BlockedCounts,
} from "./counts.js";

type ProtectionLevel = "standard" | "strict";

const DEFAULT_LEVEL: ProtectionLevel = "standard";
const DEFAULT_EXCEPTIONS: unknown[] = [];
const DEFAULT_BLOCKED_COUNTS = {};
let recordMatchQueue = Promise.resolve();

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

chrome.declarativeNetRequest.onRuleMatchedDebug.addListener(({ request }) => {
  recordMatchQueue = recordMatchQueue.then(async () => {
    const state = await chrome.storage.local.get("blockedCounts");
    const counts = (state.blockedCounts as BlockedCounts | undefined) ?? {};
    await chrome.storage.local.set({
      blockedCounts: recordDebugRuleMatch(counts, request),
    });
  });
});
