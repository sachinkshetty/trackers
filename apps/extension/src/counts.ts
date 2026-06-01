import type { TrackerCategory } from "./rules.js";

export type CategoryCounts = Partial<Record<TrackerCategory, number>>;
export type BlockedCounts = Record<string, CategoryCounts>;

export interface SiteCountSummary {
  total: number;
  categories: CategoryCounts;
}

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
