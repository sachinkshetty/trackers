import assert from "node:assert/strict";
import test from "node:test";

import {
  buildBlockedCountSummary,
  incrementBlockedCount,
  recordDebugRuleMatch,
  summarizeSiteCounts,
} from "../dist/counts.js";

test("blocked counts aggregate locally by site and category", () => {
  let counts = {};
  counts = incrementBlockedCount(counts, "news.example", "analytics");
  counts = incrementBlockedCount(counts, "news.example", "analytics");
  counts = incrementBlockedCount(counts, "news.example", "advertising");

  assert.deepEqual(summarizeSiteCounts(counts, "news.example"), {
    total: 3,
    categories: {
      advertising: 1,
      analytics: 2,
    },
  });
});

test("unknown site has an empty popup summary", () => {
  assert.deepEqual(summarizeSiteCounts({}, "news.example"), {
    total: 0,
    categories: {},
  });
});

test("debug rule matches increment total and initiator site analytics counts", () => {
  const counts = recordDebugRuleMatch({}, {
    initiator: "https://news.example",
    url: "https://analytics.example/pixel.gif",
  });

  assert.deepEqual(buildBlockedCountSummary(counts), {
    total: 1,
    sites: [
      {
        site: "news.example",
        total: 1,
        categories: { analytics: 1 },
      },
    ],
  });
});

test("debug rule matches without an initiator still increment the total", () => {
  const counts = recordDebugRuleMatch({}, {
    url: "https://analytics.example/pixel.gif",
  });

  assert.deepEqual(buildBlockedCountSummary(counts), {
    total: 1,
    sites: [],
  });
});
