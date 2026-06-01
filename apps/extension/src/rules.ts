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

export type ProtectionLevel = "standard" | "strict";

export const STRICT_MODE_WARNING =
  "Strict mode may break site functionality. Pause protection for a site if needed.";

export function buildStandardRules(trackers: TrackerRule[]): ManifestRule[] {
  return buildRules(trackers, 1);
}

export function buildRulesForLevel(
  level: ProtectionLevel,
  standardTrackers: TrackerRule[],
  strictOnlyTrackers: TrackerRule[],
): ManifestRule[] {
  const standardRules = buildStandardRules(standardTrackers);
  if (level === "standard") {
    return standardRules;
  }
  return [...standardRules, ...buildRules(strictOnlyTrackers, 100001)];
}

function buildRules(trackers: TrackerRule[], firstId: number): ManifestRule[] {
  return [...trackers]
    .sort((left, right) => left.domain.localeCompare(right.domain))
    .map((tracker, index) => ({
      id: firstId + index,
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
