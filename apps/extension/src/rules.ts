export type TrackerCategory = "advertising" | "analytics" | "social" | "other";

export interface TrackerRule {
  domain: string;
  category: TrackerCategory;
}

export interface ManifestRule {
  id: number;
  priority: number;
  action: { type: "block" };
  condition: {
    urlFilter: string;
    resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"];
  };
}

export function buildStandardRules(trackers: TrackerRule[]): ManifestRule[] {
  return [...trackers]
    .sort((left, right) => left.domain.localeCompare(right.domain))
    .map((tracker, index) => ({
      id: index + 1,
      priority: 1,
      action: { type: "block" },
      condition: {
        urlFilter: `||${tracker.domain}^`,
        resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
      },
    }));
}

export function shouldBlockStandardRequest(
  requestUrl: string,
  trackers: TrackerRule[],
): boolean {
  const hostname = new URL(requestUrl).hostname;
  return trackers.some(
    ({ domain }) => hostname === domain || hostname.endsWith(`.${domain}`),
  );
}
