import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const EASYPRIVACY_URL = "https://easylist.to/easylist/easyprivacy.txt";
const MAX_STATIC_RULES = 30000;
const root = join(dirname(fileURLToPath(import.meta.url)), "..");

export function parseEasyPrivacyDomains(subscription) {
  const domains = new Set();

  for (const rawLine of subscription.split(/\r?\n/)) {
    const line = rawLine.trim();
    const match = /^\|\|([a-z0-9.-]+)\^$/i.exec(line);
    if (!match) {
      continue;
    }

    domains.add(match[1].toLowerCase());
  }

  return [...domains].sort();
}

export function buildManifestRules(domains, startId = 1) {
  return domains.map((domain, index) => ({
    id: startId + index,
    priority: 1,
    action: { type: "block" },
    condition: {
      urlFilter: `||${domain}^`,
      resourceTypes: ["script", "image", "xmlhttprequest", "sub_frame"],
    },
  }));
}

export function buildManifestRuleShards(domains, shardSize = MAX_STATIC_RULES) {
  const shards = [];

  for (let offset = 0; offset < domains.length; offset += shardSize) {
    shards.push(buildManifestRules(domains.slice(offset, offset + shardSize), offset + 1));
  }

  return shards;
}

export function buildDesktopBundle(domains, { bundleVersion, generatedAt }) {
  return {
    schema_version: 1,
    bundle_version: bundleVersion,
    generated_at: generatedAt,
    sources: [
      {
        id: "easyprivacy",
        name: "EasyPrivacy",
        url: EASYPRIVACY_URL,
        license: "CC-BY-SA-3.0-or-later",
        attribution: "The EasyList authors (https://easylist.to/)",
      },
    ],
    rules: domains.map((domain) => ({
      id: `easyprivacy:${domain}`,
      domain,
      category: "analytics",
      confidence: "high",
      source_id: "easyprivacy",
    })),
  };
}

async function importEasyPrivacy() {
  const response = await fetch(EASYPRIVACY_URL);
  if (!response.ok) {
    throw new Error(`EasyPrivacy download failed: ${response.status} ${response.statusText}`);
  }

  const subscription = await response.text();
  const domains = parseEasyPrivacyDomains(subscription);
  const generatedAt = new Date().toISOString();
  const bundleVersion = generatedAt.slice(0, 10).replaceAll("-", ".");
  const desktopRulesDirectory = join(root, "..", "desktop", "src-tauri", "rules");
  const extensionRulesDirectory = join(root, "rules");
  const ruleShards = buildManifestRuleShards(domains);
  const manifestPath = join(root, "manifest.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));

  mkdirSync(desktopRulesDirectory, { recursive: true });
  for (const fileName of readdirSync(extensionRulesDirectory)) {
    if (/^easyprivacy-\d+\.json$/.test(fileName)) {
      rmSync(join(extensionRulesDirectory, fileName));
    }
  }
  manifest.declarative_net_request.rule_resources = ruleShards.map((rules, index) => {
    const shardNumber = index + 1;
    const fileName = `easyprivacy-${shardNumber}.json`;
    writeFileSync(
      join(extensionRulesDirectory, fileName),
      `${JSON.stringify(rules, null, 2)}\n`,
    );
    return {
      id: `easyprivacy_rules_${shardNumber}`,
      enabled: true,
      path: `rules/${fileName}`,
    };
  });
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  writeFileSync(
    join(desktopRulesDirectory, "easyprivacy.bundle.json"),
    `${JSON.stringify(buildDesktopBundle(domains, { bundleVersion, generatedAt }), null, 2)}\n`,
  );

  console.log(`Imported ${domains.length} EasyPrivacy domain rules in ${ruleShards.length} shards.`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await importEasyPrivacy();
}
