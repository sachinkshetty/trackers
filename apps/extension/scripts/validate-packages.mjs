import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const expectedPermissions = [
  "declarativeNetRequest",
  "declarativeNetRequestFeedback",
  "storage",
];
const requiredFiles = [
  "manifest.json",
  "dist/service-worker.js",
];

export function validatePackageFiles(browser) {
  const packageRoot = join(root, "packages", browser);
  const manifest = JSON.parse(
    readFileSync(join(packageRoot, "manifest.json"), "utf8"),
  );
  const ruleFiles = manifest.declarative_net_request.rule_resources.map(
    ({ path }) => path,
  );
  const popupFiles = manifest.action.default_popup
    ? [manifest.action.default_popup]
    : [];
  return {
    browser,
    manifestVersion: manifest.manifest_version,
    requiredPermissions: manifest.permissions,
    missingFiles: [...requiredFiles, ...ruleFiles, ...popupFiles].filter(
      (relativePath) => !existsSync(join(packageRoot, relativePath)),
    ),
  };
}

export function validatePackages() {
  const results = ["chrome", "edge"].map(validatePackageFiles);
  for (const result of results) {
    if (
      result.manifestVersion !== 3 ||
      JSON.stringify(result.requiredPermissions) !==
        JSON.stringify(expectedPermissions) ||
      result.missingFiles.length > 0
    ) {
      throw new Error(`invalid ${result.browser} package: ${JSON.stringify(result)}`);
    }
  }
  return results;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  console.log(JSON.stringify(validatePackages(), null, 2));
}

