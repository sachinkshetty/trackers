import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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
      </div>
    </section>
  </main>
`;

const status = app.querySelector('[data-status]');
const startButton = app.querySelector('[data-start]');
const cancelButton = app.querySelector('[data-cancel]');
const profilesContainer = app.querySelector('[data-profiles]');
const progressContainer = app.querySelector('[data-progress]');
const warningsContainer = app.querySelector('[data-warnings]');

const state = {
  discovery: null,
  progress: [],
  warnings: [],
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

function setRunning(running) {
  state.scanRunning = running;
  startButton.disabled = running || !state.discovery;
  cancelButton.disabled = !running;
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

function registerListeners() {
  listen('scan-progress', (event) => {
    state.progress.push(event.payload);
    renderProgress();
    setStatus(`Scanning ${event.payload.profileName} on ${event.payload.browser}.`);
  });

  listen('scan-complete', (event) => {
    state.warnings = [...state.warnings, ...event.payload.warnings];
    renderWarnings();
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
  state.warnings = browserWarnings(state.discovery).map((warning) => warning.message);
  renderProgress();
  renderWarnings();
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

registerListeners();
loadDiscovery().catch((error) => {
  setStatus(`Failed to load profiles: ${error}`);
});
