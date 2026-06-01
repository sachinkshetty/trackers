import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { buildLayeredResults } from './results.js';
import './style.css';

const app = document.querySelector('#app');

app.innerHTML = `
  <main class="shell">
    <section class="hero">
      <p class="eyebrow">Desktop dashboard</p>
      <h1>Trackers</h1>
      <p class="lede">
        Discover browser profiles, run a read-only scan, and watch per-profile progress live.
      </p>
    </section>

    <section class="panel">
      <div class="toolbar">
        <p class="status" data-status>Loading browser profiles...</p>
        <div class="actions">
          <button class="button" data-expert-toggle disabled>Expert view: off</button>
          <button class="button primary" data-start disabled>Start scan</button>
          <button class="button" data-cancel disabled>Cancel scan</button>
        </div>
      </div>

      <div class="grid">
        <article class="card">
          <h2>Profiles</h2>
          <div class="stack" data-profiles></div>
        </article>

        <article class="card">
          <h2>Progress</h2>
          <div class="stack" data-progress></div>
        </article>

        <article class="card">
          <h2>Warnings</h2>
          <div class="stack" data-warnings></div>
        </article>

        <article class="card results-card">
          <div class="results-header">
            <h2>Findings</h2>
            <p class="subtle" data-results-summary>No scan results yet.</p>
          </div>
          <div class="stack" data-results></div>
        </article>

        <article class="card results-card">
          <div class="results-header">
            <h2>Cleanup preview</h2>
            <p class="subtle" data-cleanup-summary>No cleanup plan yet.</p>
          </div>
          <div class="toolbar cleanup-toolbar">
            <div class="actions">
              <button class="button primary" data-cleanup-mode="review">Review</button>
              <button class="button" data-cleanup-mode="balanced">Balanced</button>
              <button class="button" data-cleanup-mode="aggressive">Aggressive</button>
            </div>
            <label class="confirm-toggle">
              <input type="checkbox" data-aggressive-confirm />
              Confirm aggressive cleanup
            </label>
            <button class="button primary" data-preview-cleanup disabled>Preview cleanup</button>
          </div>
          <div class="stack" data-cleanup-preview></div>
        </article>
      </div>
    </section>
  </main>
`;

const status = app.querySelector('[data-status]');
const expertToggle = app.querySelector('[data-expert-toggle]');
const startButton = app.querySelector('[data-start]');
const cancelButton = app.querySelector('[data-cancel]');
const profilesContainer = app.querySelector('[data-profiles]');
const progressContainer = app.querySelector('[data-progress]');
const warningsContainer = app.querySelector('[data-warnings]');
const resultsContainer = app.querySelector('[data-results]');
const resultsSummary = app.querySelector('[data-results-summary]');
const cleanupSummary = app.querySelector('[data-cleanup-summary]');
const cleanupPreviewContainer = app.querySelector('[data-cleanup-preview]');
const previewCleanupButton = app.querySelector('[data-preview-cleanup]');
const aggressiveConfirm = app.querySelector('[data-aggressive-confirm]');
const cleanupModeButtons = app.querySelectorAll('[data-cleanup-mode]');

const state = {
  discovery: null,
  progress: [],
  warnings: [],
  scanResult: null,
  expertMode: false,
  cleanupMode: 'review',
  cleanupPreview: null,
  scanRunning: false,
};

function browserProfiles(snapshot) {
  return [...snapshot.chrome.profiles, ...snapshot.edge.profiles];
}

function browserWarnings(snapshot) {
  return [...snapshot.chrome.warnings, ...snapshot.edge.warnings];
}

function renderList(container, items, emptyLabel) {
  if (items.length === 0) {
    container.innerHTML = `<p class="empty">${emptyLabel}</p>`;
    return;
  }

  container.innerHTML = items
    .map((item) => `<div class="row">${item}</div>`)
    .join('');
}

function renderProfiles() {
  if (!state.discovery) {
    renderList(profilesContainer, [], 'Waiting for discovery.');
    return;
  }

  const items = browserProfiles(state.discovery).map(
    (profile) => `${profile.browser} · ${profile.profile_name}`,
  );
  renderList(profilesContainer, items, 'No profiles found.');
}

function renderProgress() {
  const items = state.progress.map(
    (event) =>
      `${event.browser} · ${event.profileName} (${event.completedProfiles}/${event.totalProfiles})`,
  );
  renderList(progressContainer, items, 'No scan progress yet.');
}

function renderWarnings() {
  renderList(warningsContainer, state.warnings, 'No warnings yet.');
}

function renderResults() {
  if (!state.scanResult) {
    resultsSummary.textContent = 'No scan results yet.';
    resultsContainer.innerHTML = '<p class="empty">Run a scan to inspect layered findings.</p>';
    return;
  }

  const model = buildLayeredResults(state.scanResult, { expertMode: state.expertMode });
  const browserCount = model.browsers.length;
  const profileCount = model.browsers.reduce(
    (total, browser) => total + browser.profiles.length,
    0,
  );
  const findingCount = model.browsers.reduce(
    (browserTotal, browser) =>
      browserTotal +
      browser.profiles.reduce(
        (profileTotal, profile) =>
          profileTotal +
          profile.groups.reduce(
            (groupTotal, group) =>
              groupTotal +
              group.artifacts.reduce((artifactTotal, artifact) => artifactTotal + artifact.findings.length, 0),
            0,
          ),
        0,
      ),
    0,
  );

  resultsSummary.textContent = `${browserCount} browser(s), ${profileCount} profile(s), ${findingCount} finding(s).`;
  resultsContainer.innerHTML = model.browsers
    .map(
      (browser) => `
        <details open class="browser-group">
          <summary>${browser.browser} - ${browser.profiles.length} profile(s)</summary>
          <div class="stack nested">
            ${browser.profiles
              .map(
                (profile) => `
                  <details class="profile-group">
                    <summary>${profile.profileName} - ${profile.groups.length} site group(s)</summary>
                    <div class="stack nested">
                      ${profile.warnings.length > 0 ? `<div class="row warning">${profile.warnings.join('<br />')}</div>` : ''}
                      ${profile.groups
                        .map(
                          (group) => `
                            <details class="site-group">
                              <summary>${group.site ?? '(no site)'} - ${group.artifacts.length} artifact group(s)</summary>
                              <div class="stack nested">
                                ${group.artifacts
                                  .map(
                                    (artifact) => `
                                      <div class="row artifact">
                                        <div class="artifact-head">
                                          <strong>${artifact.artifactType}</strong>
                                          <span>${artifact.confidence ?? 'unknown confidence'}</span>
                                          <span>${artifact.cleanupImpact}</span>
                                        </div>
                                        ${state.expertMode && artifact.evidenceSummary ? `<p class="artifact-detail">${artifact.evidenceSummary}</p>` : ''}
                                        ${state.expertMode && artifact.profilePath ? `<p class="artifact-detail">${artifact.profilePath}</p>` : ''}
                                      </div>
                                    `,
                                  )
                                  .join('')}
                              </div>
                            </details>
                          `,
                        )
                        .join('')}
                    </div>
                  </details>
                `,
              )
              .join('')}
          </div>
        </details>
      `,
    )
    .join('');
}

function renderCleanupPreview() {
  if (!state.cleanupPreview) {
    cleanupSummary.textContent = 'No cleanup plan yet.';
    cleanupPreviewContainer.innerHTML =
      '<p class="empty">Preview a cleanup mode to review actions and locked-item choices.</p>';
    return;
  }

  const { plan, lockedActionIds, warnings, requiresConfirmation } = state.cleanupPreview;
  cleanupSummary.textContent = `${plan.mode} mode · ${plan.actions.length} action(s) · ${lockedActionIds.length} locked`;

  cleanupPreviewContainer.innerHTML = `
    <div class="row">
      <strong>Warnings</strong>
      <div class="stack nested">
        ${warnings.length === 0 ? '<p class="empty">No cleanup warnings.</p>' : warnings.map((warning) => `<p class="row warning">${warning}</p>`).join('')}
      </div>
    </div>
    <div class="row">
      <strong>Locked items</strong>
      <p class="subtle">${requiresConfirmation ? 'Aggressive cleanup needs explicit confirmation before execution.' : 'Locked actions require browser closure before execution.'}</p>
      <div class="stack nested">
        ${lockedActionIds.length === 0 ? '<p class="empty">No locked actions in this plan.</p>' : lockedActionIds.map((id) => `<p class="row">${id}</p>`).join('')}
      </div>
    </div>
    <div class="row">
      <strong>Planned actions</strong>
      <div class="stack nested">
        ${plan.actions
          .map(
            (action) => `
              <div class="row artifact">
                <div class="artifact-head">
                  <strong>${action.id}</strong>
                  <span>${action.artifactType}</span>
                  <span>${action.requiresBrowserClosed ? 'browser close required' : 'ready'}</span>
                </div>
                <p class="artifact-detail">${action.target.kind}</p>
              </div>
            `,
          )
          .join('')}
      </div>
    </div>
  `;
}

function syncCleanupControls() {
  aggressiveConfirm.disabled = state.cleanupMode !== 'aggressive' || state.scanRunning;
  previewCleanupButton.disabled =
    state.scanRunning ||
    !state.scanResult ||
    (state.cleanupMode === 'aggressive' && !aggressiveConfirm.checked);
}

function setRunning(running) {
  state.scanRunning = running;
  startButton.disabled = running || !state.discovery;
  cancelButton.disabled = !running;
  syncCleanupControls();
}

function setStatus(message) {
  status.textContent = message;
}

function combineDiscovery(snapshot) {
  return {
    profiles: browserProfiles(snapshot),
    warnings: browserWarnings(snapshot),
  };
}

function allFindingIds(scanResult) {
  return (scanResult?.profiles ?? []).flatMap((profile) =>
    profile.findings.map((finding) => finding.id),
  );
}

function flattenWarnings(scanResult) {
  const discoveryWarnings = state.discovery ? browserWarnings(state.discovery).map((warning) => warning.message) : [];
  const profileWarnings = (scanResult?.profiles ?? []).flatMap((profile) => profile.warnings ?? []);
  return [...discoveryWarnings, ...profileWarnings];
}

function registerListeners() {
  void listen('scan-progress', (event) => {
    state.progress.push(event.payload);
    renderProgress();
    setStatus(`Scanning ${event.payload.profileName} on ${event.payload.browser}.`);
  });

  void listen('scan-complete', (event) => {
    state.scanResult = event.payload;
    state.warnings = flattenWarnings(event.payload);
    renderWarnings();
    renderResults();
    state.cleanupPreview = null;
    renderCleanupPreview();
    setRunning(false);
    setStatus(
      event.payload.cancelled
        ? `Scan cancelled after ${event.payload.completedProfiles} profile(s).`
        : `Scan finished across ${event.payload.completedProfiles} profile(s).`,
    );
  });
}

async function loadDiscovery() {
  const snapshot = await invoke('discover_profiles', {});
  state.discovery = snapshot;
  renderProfiles();
  state.warnings = browserWarnings(snapshot).map((warning) => warning.message);
  renderWarnings();
  renderResults();
  renderCleanupPreview();
  setStatus(
    browserProfiles(snapshot).length > 0
      ? `Discovered ${browserProfiles(snapshot).length} profile(s). Ready to scan.`
      : 'No browser profiles found.',
  );
  setRunning(false);
}

async function startScan() {
  if (!state.discovery || state.scanRunning) {
    return;
  }

  state.progress = [];
  state.scanResult = null;
  state.cleanupPreview = null;
  state.warnings = browserWarnings(state.discovery).map((warning) => warning.message);
  renderProgress();
  renderWarnings();
  renderResults();
  renderCleanupPreview();
  setRunning(true);
  setStatus('Starting read-only scan...');

  await invoke('start_scan', {
    request: {
      discovery: combineDiscovery(state.discovery),
    },
  });
}

async function cancelScan() {
  if (!state.scanRunning) {
    return;
  }

  await invoke('cancel_scan');
  setStatus('Cancellation requested.');
}

async function previewCleanup() {
  if (!state.scanResult) {
    return;
  }

  const preview = await invoke('preview_cleanup', {
    request: {
      scanResult: state.scanResult,
      mode: state.cleanupMode,
      selectedFindingIds: allFindingIds(state.scanResult),
      aggressiveConfirmed: state.cleanupMode !== 'aggressive' || aggressiveConfirm.checked,
    },
  });

  state.cleanupPreview = preview;
  renderCleanupPreview();
  setStatus(`Cleanup preview ready in ${state.cleanupMode} mode.`);
}

function toggleExpertMode() {
  state.expertMode = !state.expertMode;
  expertToggle.textContent = state.expertMode ? 'Expert view: on' : 'Expert view: off';
  renderResults();
}

function setCleanupMode(mode) {
  state.cleanupMode = mode;
  cleanupModeButtons.forEach((button) => {
    const active = button.dataset.cleanupMode === mode;
    button.classList.toggle('primary', active);
  });
  if (mode !== 'aggressive') {
    aggressiveConfirm.checked = false;
  }
  state.cleanupPreview = null;
  renderCleanupPreview();
  syncCleanupControls();
}

startButton.addEventListener('click', () => {
  startScan().catch((error) => {
    setRunning(false);
    setStatus(`Scan failed: ${error}`);
  });
});

cancelButton.addEventListener('click', () => {
  cancelScan().catch((error) => {
    setStatus(`Cancel failed: ${error}`);
  });
});

expertToggle.addEventListener('click', toggleExpertMode);

previewCleanupButton.addEventListener('click', () => {
  previewCleanup().catch((error) => {
    setStatus(`Cleanup preview failed: ${error}`);
  });
});

cleanupModeButtons.forEach((button) => {
  button.addEventListener('click', () => {
    setCleanupMode(button.dataset.cleanupMode);
  });
});

aggressiveConfirm.addEventListener('change', () => {
  syncCleanupControls();
  state.cleanupPreview = null;
  renderCleanupPreview();
});

registerListeners();
setCleanupMode('review');
loadDiscovery().catch((error) => {
  setStatus(`Failed to load profiles: ${error}`);
});
