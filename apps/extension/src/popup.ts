import {
  buildBlockedCountSummary,
  summarizeSiteCounts,
  type BlockedCounts,
} from "./counts.js";

export async function loadCurrentSiteSummary(site: string) {
  const state = await chrome.storage.local.get("blockedCounts");
  return summarizeSiteCounts(
    (state.blockedCounts as BlockedCounts | undefined) ?? {},
    site,
  );
}

async function renderPopup() {
  const state = await chrome.storage.local.get("blockedCounts");
  const summary = buildBlockedCountSummary(
    (state.blockedCounts as BlockedCounts | undefined) ?? {},
  );
  const total = document.querySelector("#total");
  const sites = document.querySelector("#sites");

  if (total) {
    total.textContent = String(summary.total);
  }
  if (sites) {
    sites.replaceChildren(
      ...summary.sites.slice(0, 10).map(({ site, total: siteTotal }) => {
        const row = document.createElement("li");
        row.textContent = `${site}: ${siteTotal}`;
        return row;
      }),
    );
  }
}

document.querySelector("#clear")?.addEventListener("click", async () => {
  await chrome.storage.local.set({ blockedCounts: {} });
  await renderPopup();
});

void renderPopup();
