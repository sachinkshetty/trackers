import type { TrackerCategory } from "./rules.js";

export type CategoryCounts = Partial<Record<TrackerCategory, number>>;
export type BlockedCounts = Record<string, CategoryCounts>;

export interface SiteCountSummary {
  total: number;
  categories: CategoryCounts;
}

export interface BlockedCountSummary {
  total: number;
  sites: Array<SiteCountSummary & { site: string }>;
}

export interface DebugMatchedRequest {
  initiator?: string;
  url: string;
}

const TOTAL_SITE = "__total__";

export function incrementBlockedCount(
  counts: BlockedCounts,
  site: string,
  category: TrackerCategory,
): BlockedCounts {
  const categories = counts[site] ?? {};
  return {
    ...counts,
    [site]: {
      ...categories,
      [category]: (categories[category] ?? 0) + 1,
    },
  };
}

export function summarizeSiteCounts(
  counts: BlockedCounts,
  site: string,
): SiteCountSummary {
  const categories = counts[site] ?? {};
  return {
    total: Object.values(categories).reduce(
      (sum, count) => sum + (count ?? 0),
      0,
    ),
    categories,
  };
}

export function recordDebugRuleMatch(
  counts: BlockedCounts,
  request: DebugMatchedRequest,
): BlockedCounts {
  const updatedCounts = incrementBlockedCount(counts, TOTAL_SITE, "analytics");
  if (!request.initiator) {
    return updatedCounts;
  }

  try {
    return incrementBlockedCount(
      updatedCounts,
      new URL(request.initiator).hostname,
      "analytics",
    );
  } catch {
    return updatedCounts;
  }
}

export function buildBlockedCountSummary(counts: BlockedCounts): BlockedCountSummary {
  return {
    total: summarizeSiteCounts(counts, TOTAL_SITE).total,
    sites: Object.keys(counts)
      .filter((site) => site !== TOTAL_SITE)
      .map((site) => ({ site, ...summarizeSiteCounts(counts, site) }))
      .sort((left, right) => right.total - left.total || left.site.localeCompare(right.site)),
  };
}
