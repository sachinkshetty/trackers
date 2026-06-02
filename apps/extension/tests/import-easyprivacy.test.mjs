import assert from "node:assert/strict";
import test from "node:test";

import {
  buildDesktopBundle,
  buildManifestRuleShards,
  buildManifestRules,
  parseEasyPrivacyDomains,
} from "../scripts/import-easyprivacy.mjs";

const fixture = `
! EasyPrivacy fixture
||analytics.example^
||PIXEL.EXAMPLE^
||analytics.example^
||path.example/tracker.js
||option.example^$third-party
@@||allow.example^
##.cosmetic-rule
`;

test("EasyPrivacy importer keeps only unique pure domain block rules", () => {
  assert.deepEqual(parseEasyPrivacyDomains(fixture), [
    "analytics.example",
    "pixel.example",
  ]);
});

test("EasyPrivacy importer emits Manifest V3 block rules", () => {
  assert.deepEqual(buildManifestRules(["analytics.example"]), [
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
});

test("EasyPrivacy importer shards Manifest V3 rules without resetting rule ids", () => {
  assert.deepEqual(
    buildManifestRuleShards(["a.example", "b.example", "c.example"], 2).map(
      (rules) => rules.map(({ id }) => id),
    ),
    [[1, 2], [3]],
  );
});

test("EasyPrivacy importer emits desktop provenance metadata", () => {
  const bundle = buildDesktopBundle(["analytics.example"], {
    bundleVersion: "2026.06.02.1",
    generatedAt: "2026-06-02T00:00:00Z",
  });

  assert.equal(bundle.sources[0].id, "easyprivacy");
  assert.equal(bundle.rules[0].domain, "analytics.example");
  assert.equal(bundle.rules[0].source_id, "easyprivacy");
});
