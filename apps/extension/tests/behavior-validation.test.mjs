import assert from "node:assert/strict";
import test from "node:test";

import {
  incrementBlockedCount,
  summarizeSiteCounts,
} from "../dist/counts.js";
import {
  isSitePaused,
  pauseSitePermanently,
  pauseSiteTemporarily,
} from "../dist/exceptions.js";
import {
  STRICT_MODE_WARNING,
  buildRulesForLevel,
  shouldBlockStandardRequest,
} from "../dist/rules.js";

const standardTrackers = [
  { domain: "analytics.example", category: "analytics" },
  { domain: "ads.example", category: "advertising" },
];

const strictOnlyTrackers = [
  { domain: "widgets.example", category: "other" },
];

test("standard and strict protection, counts, pauses, and exceptions behave locally", () => {
  const chromeStandardRules = buildRulesForLevel(
    "standard",
    standardTrackers,
    strictOnlyTrackers,
  );
  const chromeStrictRules = buildRulesForLevel(
    "strict",
    standardTrackers,
    strictOnlyTrackers,
  );
  const edgeStandardRules = buildRulesForLevel(
    "standard",
    standardTrackers,
    strictOnlyTrackers,
  );

  assert.equal(shouldBlockStandardRequest("https://cdn.analytics.example/pixel.js", standardTrackers), true);
  assert.equal(shouldBlockStandardRequest("https://news.example/article", standardTrackers), false);
  assert.equal(chromeStandardRules.length, 2);
  assert.equal(chromeStrictRules.length, 3);
  assert.equal(edgeStandardRules.length, 2);
  assert.match(STRICT_MODE_WARNING, /break/i);

  let counts = {};
  counts = incrementBlockedCount(counts, "news.example", "analytics");
  counts = incrementBlockedCount(counts, "news.example", "advertising");
  counts = incrementBlockedCount(counts, "news.example", "advertising");

  assert.deepEqual(summarizeSiteCounts(counts, "news.example"), {
    total: 3,
    categories: {
      advertising: 2,
      analytics: 1,
    },
  });

  const now = 1_000;
  const permanentPause = pauseSitePermanently([], "news.example");
  const temporaryPause = pauseSiteTemporarily([], "news.example", now, 500);

  assert.equal(isSitePaused(permanentPause, "news.example", now), true);
  assert.equal(isSitePaused(temporaryPause, "news.example", now + 499), true);
  assert.equal(isSitePaused(temporaryPause, "news.example", now + 500), false);
  assert.equal(isSitePaused(temporaryPause, "other.example", now + 500), false);
});
