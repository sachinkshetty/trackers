import assert from "node:assert/strict";
import test from "node:test";

import {
  isSitePaused,
  pauseSitePermanently,
  pauseSiteTemporarily,
} from "../dist/exceptions.js";

test("permanent exception pauses protection for the selected site", () => {
  const exceptions = pauseSitePermanently([], "news.example");

  assert.equal(isSitePaused(exceptions, "news.example", 1000), true);
  assert.equal(isSitePaused(exceptions, "other.example", 1000), false);
});

test("temporary exception expires locally", () => {
  const exceptions = pauseSiteTemporarily([], "news.example", 1000, 500);

  assert.equal(isSitePaused(exceptions, "news.example", 1499), true);
  assert.equal(isSitePaused(exceptions, "news.example", 1500), false);
  assert.deepEqual(JSON.parse(JSON.stringify(exceptions)), [
    { site: "news.example", expiresAt: 1500 },
  ]);
});
