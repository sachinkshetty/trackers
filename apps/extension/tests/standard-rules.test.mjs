import assert from "node:assert/strict";
import test from "node:test";

import {
  buildStandardRules,
  shouldBlockStandardRequest,
} from "../dist/rules.js";

const trackers = [
  { domain: "analytics.example", category: "analytics" },
  { domain: "ads.example", category: "advertising" },
];

test("standard rules compile to Manifest V3 block rules", () => {
  assert.deepEqual(buildStandardRules(trackers), [
    {
      id: 1,
      priority: 1,
      action: { type: "block" },
      condition: {
        urlFilter: "||ads.example^",
        resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
      },
    },
    {
      id: 2,
      priority: 1,
      action: { type: "block" },
      condition: {
        urlFilter: "||analytics.example^",
        resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
      },
    },
  ]);
});

test("controlled fixture requests block known trackers only", () => {
  assert.equal(
    shouldBlockStandardRequest("https://cdn.analytics.example/pixel.js", trackers),
    true,
  );
  assert.equal(
    shouldBlockStandardRequest("https://news.example/article", trackers),
    false,
  );
});
