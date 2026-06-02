import assert from 'node:assert/strict';
import test from 'node:test';

import {
  cleanupArtifactImpact,
  cleanupScopeLabel,
} from '../src/cleanup-preview.js';

test('cleanup artifact impact explains sign-out and preference loss risks', () => {
  assert.equal(
    cleanupArtifactImpact('cookie', 'may_sign_out'),
    'Cookies can sign you out of websites and reset login sessions.',
  );
  assert.equal(
    cleanupArtifactImpact('local_storage', 'may_remove_preferences'),
    'Local Storage can remove saved preferences, settings, and site state.',
  );
});

test('cleanup artifact impact explains offline and reload impact', () => {
  assert.equal(
    cleanupArtifactImpact('indexed_db', 'review_required'),
    'IndexedDB can remove offline app data and stored records that websites use to resume state.',
  );
  assert.equal(
    cleanupArtifactImpact('service_worker', 'review_required'),
    'Service workers can affect offline support, background sync, and push notifications.',
  );
  assert.equal(
    cleanupArtifactImpact('cache', 'review_required'),
    'Cache cleanup can force slower reloads and re-download site assets.',
  );
  assert.equal(
    cleanupArtifactImpact('history', 'review_required'),
    'History cleanup removes browsing history, search suggestions, and visit traces.',
  );
});

test('cleanup scope label distinguishes tracker cleanup from general privacy cleanup', () => {
  assert.equal(cleanupScopeLabel(false), 'Tracker cleanup');
  assert.equal(cleanupScopeLabel(true), 'General privacy cleanup');
});
