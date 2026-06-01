import { summarizeSiteCounts, type BlockedCounts } from "./counts.js";

export async function loadCurrentSiteSummary(site: string) {
  const state = await chrome.storage.local.get("blockedCounts");
  return summarizeSiteCounts(
    (state.blockedCounts as BlockedCounts | undefined) ?? {},
    site,
  );
}
