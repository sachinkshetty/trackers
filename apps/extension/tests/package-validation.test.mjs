import assert from "node:assert/strict";
import test from "node:test";

import { validatePackageFiles } from "../scripts/validate-packages.mjs";

test("Chrome and Edge packages contain required local extension assets", () => {
  for (const browser of ["chrome", "edge"]) {
    assert.deepEqual(validatePackageFiles(browser), {
      browser,
      manifestVersion: 3,
      requiredPermissions: ["declarativeNetRequest", "storage"],
      missingFiles: [],
    });
  }
});
