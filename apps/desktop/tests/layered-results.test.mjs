import assert from 'node:assert/strict';
import test from 'node:test';

import { buildLayeredResults } from '../src/results.js';

test('layered results group findings by browser profile site artifact and confidence', () => {
  const scanResult = {
    profiles: [
      {
        browser: 'chrome',
        profileName: 'Default',
        profilePath: 'C:\\Chrome\\User Data\\Default',
        findings: [
          {
            id: 'cookie:analytics.example',
            profile: {
              browser: 'chrome',
              profile_name: 'Default',
              profile_path: 'C:\\Chrome\\User Data\\Default',
            },
            artifact_type: 'cookie',
            site: 'analytics.example',
            evidence_summary: 'cookie host matched tracker rule',
            confidence: 'high',
            cleanup_impact: 'may_sign_out',
          },
          {
            id: 'cookie:cdn.analytics.example',
            profile: {
              browser: 'chrome',
              profile_name: 'Default',
              profile_path: 'C:\\Chrome\\User Data\\Default',
            },
            artifact_type: 'cookie',
            site: 'cdn.analytics.example',
            evidence_summary: 'cookie host matched tracker rule',
            confidence: 'low',
            cleanup_impact: 'may_sign_out',
          },
        ],
        warnings: [],
      },
      {
        browser: 'edge',
        profileName: 'Profile 1',
        profilePath: 'C:\\Edge\\User Data\\Profile 1',
        findings: [
          {
            id: 'setting:homepage',
            profile: {
              browser: 'edge',
              profile_name: 'Profile 1',
              profile_path: 'C:\\Edge\\User Data\\Profile 1',
            },
            artifact_type: 'setting',
            site: null,
            evidence_summary: 'privacy setting homepage is exposed as https://example.test/',
            confidence: null,
            cleanup_impact: 'review_required',
          },
        ],
        warnings: [],
      },
    ],
  };

  const grouped = buildLayeredResults(scanResult, { expertMode: false });

  assert.deepEqual(grouped.browsers.map((browser) => browser.browser), ['chrome', 'edge']);
  assert.equal(grouped.browsers[0].profiles[0].groups[0].site, 'analytics.example');
  assert.equal(grouped.browsers[0].profiles[0].groups[0].artifacts[0].confidence, 'high');
  assert.ok(!('evidenceSummary' in grouped.browsers[0].profiles[0].groups[0].artifacts[0]));

  const expert = buildLayeredResults(scanResult, { expertMode: true });
  assert.equal(
    expert.browsers[0].profiles[0].groups[0].artifacts[0].evidenceSummary,
    'cookie host matched tracker rule',
  );
  assert.equal(
    expert.browsers[1].profiles[0].groups[0].artifacts[0].profilePath,
    'C:\\Edge\\User Data\\Profile 1',
  );
});
