import assert from "node:assert/strict";
import test from "node:test";

import {
  STRICT_MODE_WARNING,
  buildRulesForLevel,
} from "../dist/rules.js";

const standard = [{ domain: "analytics.example", category: "analytics" }];
const strictOnly = [{ domain: "widgets.example", category: "other" }];

test("strict mode is opt-in and adds separately identifiable rules", () => {
  assert.match(STRICT_MODE_WARNING, /break/i);
  assert.deepEqual(buildRulesForLevel("standard", standard, strictOnly), [
    {
      id: 1,
      priority: 1,
      action: { type: "block" },
      condition: {
        urlFilter: "||analytics.example^",
        resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
      },
    },
  ]);
  assert.deepEqual(buildRulesForLevel("strict", standard, strictOnly), [
    {
      id: 1,
      priority: 1,
      action: { type: "block" },
      condition: {
        urlFilter: "||analytics.example^",
        resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
      },
    },
    {
      id: 100001,
      priority: 1,
      action: { type: "block" },
      condition: {
        urlFilter: "||widgets.example^",
        resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
      },
    },
  ]);
});

test("switching back to standard removes strict-only rules", () => {
  const strictRules = buildRulesForLevel("strict", standard, strictOnly);
  const standardRules = buildRulesForLevel("standard", standard, strictOnly);

  assert.equal(strictRules.length, 2);
  assert.equal(standardRules.length, 1);
  assert.equal(standardRules.some(({ id }) => id >= 100000), false);
});
