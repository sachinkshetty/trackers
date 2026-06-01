import assert from "node:assert/strict";
import test from "node:test";

import {
  incrementBlockedCount,
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
