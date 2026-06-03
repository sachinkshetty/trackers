import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  cleanupArtifactImpact,
  cleanupImpactLabel,
  cleanupScopeLabel,
} from './cleanup-preview.js';
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
            <label class="cleanup-filter">
              Browser
              <select data-cleanup-browser>
                <option value="all">All browsers</option>
                <option value="chrome">Chrome</option>
                <option value="edge">Edge</option>
              </select>
            </label>
            <label class="cleanup-filter">
              Locked items
              <select data-lock-resolution>
                <option value="retryAfterManualClose">Retry after manual close</option>
                <option value="skipLocked">Skip locked actions</option>
                <option value="requestAutomaticClose">Request automatic close</option>
              </select>
            </label>
            <label class="confirm-toggle">
              <input type="checkbox" data-include-general-cleanup />
              Include general browser data
            </label>
            <label class="confirm-toggle">
              <input type="checkbox" data-aggressive-confirm />
              Confirm aggressive cleanup
            </label>
            <label class="confirm-toggle">
              <input type="checkbox" data-auto-close-confirm />
              Confirm automatic browser close
            </label>
            <label class="confirm-toggle">
              <input type="checkbox" data-allow-no-backup />
              Proceed without a backup
            </label>
            <button class="button primary" data-preview-cleanup disabled>Preview cleanup</button>
            <button class="button" data-execute-cleanup disabled>Clean trackers</button>
          </div>
          <div class="stack" data-cleanup-preview></div>
        </article>

        <article class="card">
          <div class="results-header">
            <h2>Settings and privacy</h2>
            <p class="subtle" data-settings-summary>Loading bundle metadata...</p>
          </div>
          <div class="stack">
            <div class="row">
              <strong>Rule bundle version</strong>
              <p class="artifact-detail" data-rule-version>Loading...</p>
            </div>
            <div class="row">
              <strong>Update state</strong>
              <p class="artifact-detail" data-update-state>Loading...</p>
            </div>
            <label class="setting-toggle">
              <input type="checkbox" data-telemetry-opt-in />
              Anonymous telemetry opt-in
            </label>
            <label class="setting-toggle">
              <input type="checkbox" data-diagnostics-opt-in />
              Diagnostics report opt-in
            </label>
            <p class="subtle">
              Telemetry remains disabled unless you opt in. Diagnostics stay separate so troubleshooting
              reports do not mix with aggregate usage metrics.
            </p>
          </div>
        </article>

        <article class="card results-card">
          <div class="results-header">
            <h2>Scheduled maintenance</h2>
            <p class="subtle" data-scheduler-summary>Loading scheduled maintenance preferences...</p>
          </div>
          <div class="stack" data-scheduler-panel>
            <div class="row">
              <strong>Rule refresh</strong>
              <label class="setting-toggle">
                <input type="checkbox" data-rule-refresh-enabled />
                Enable scheduled rule refresh
              </label>
              <label class="cleanup-filter">
                Frequency
                <select data-rule-refresh-frequency>
                  <option value="1">Every day</option>
                  <option value="7">Every 7 days</option>
                  <option value="14">Every 14 days</option>
                  <option value="30">Every 30 days</option>
                </select>
              </label>
              <p class="artifact-detail" data-rule-refresh-last-run>Last run: Loading...</p>
              <p class="artifact-detail" data-rule-refresh-next-run>Next run: Loading...</p>
              <p class="artifact-detail" data-rule-refresh-last-result>Last result: Loading...</p>
            </div>
            <div class="row">
              <strong>Read-only rescan</strong>
              <label class="setting-toggle">
                <input type="checkbox" data-scheduled-rescan-enabled />
                Enable scheduled read-only rescans
              </label>
              <label class="cleanup-filter">
                Frequency
                <select data-scheduled-rescan-frequency>
                  <option value="1">Every day</option>
                  <option value="7">Every 7 days</option>
                  <option value="14">Every 14 days</option>
                  <option value="30">Every 30 days</option>
                </select>
              </label>
              <p class="artifact-detail" data-scheduled-rescan-last-run>Last run: Loading...</p>
              <p class="artifact-detail" data-scheduled-rescan-next-run>Next run: Loading...</p>
              <p class="artifact-detail" data-scheduled-rescan-last-result>Last result: Loading...</p>
            </div>
            <p class="subtle">
              Scheduled maintenance is disabled by default. When enabled, both tasks use conservative
              intervals and stay read-only until the user explicitly runs cleanup.
            </p>
          </div>
        </article>

        <article class="card results-card">
          <div class="results-header">
            <h2>Cleanup history</h2>
            <p class="subtle" data-cleanup-history-summary>No cleanup history yet.</p>
          </div>
          <div class="toolbar cleanup-toolbar">
            <button class="button" data-clear-cleanup-history disabled>Clear history</button>
          </div>
          <div class="stack" data-cleanup-history></div>
        </article>

        <article class="card results-card">
          <div class="results-header">
            <h2>Restore backups</h2>
            <p class="subtle" data-restore-summary>No restore preview yet.</p>
          </div>
          <div class="toolbar cleanup-toolbar">
            <button class="button" data-preview-restore>Preview restore</button>
            <button class="button primary" data-execute-restore disabled>Restore latest cleanup</button>
          </div>
          <div class="stack" data-restore-preview></div>
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
const executeCleanupButton = app.querySelector('[data-execute-cleanup]');
const aggressiveConfirm = app.querySelector('[data-aggressive-confirm]');
const autoCloseConfirm = app.querySelector('[data-auto-close-confirm]');
const includeGeneralCleanup = app.querySelector('[data-include-general-cleanup]');
const cleanupModeButtons = app.querySelectorAll('[data-cleanup-mode]');
const cleanupBrowser = app.querySelector('[data-cleanup-browser]');
const lockResolution = app.querySelector('[data-lock-resolution]');
const allowNoBackup = app.querySelector('[data-allow-no-backup]');
const settingsSummary = app.querySelector('[data-settings-summary]');
const ruleVersion = app.querySelector('[data-rule-version]');
const updateState = app.querySelector('[data-update-state]');
const telemetryOptIn = app.querySelector('[data-telemetry-opt-in]');
const diagnosticsOptIn = app.querySelector('[data-diagnostics-opt-in]');
const schedulerSummary = app.querySelector('[data-scheduler-summary]');
const schedulerPanel = app.querySelector('[data-scheduler-panel]');
const ruleRefreshEnabled = app.querySelector('[data-rule-refresh-enabled]');
const ruleRefreshFrequency = app.querySelector('[data-rule-refresh-frequency]');
const ruleRefreshLastRun = app.querySelector('[data-rule-refresh-last-run]');
const ruleRefreshNextRun = app.querySelector('[data-rule-refresh-next-run]');
const ruleRefreshLastResult = app.querySelector('[data-rule-refresh-last-result]');
const scheduledRescanEnabled = app.querySelector('[data-scheduled-rescan-enabled]');
const scheduledRescanFrequency = app.querySelector('[data-scheduled-rescan-frequency]');
const scheduledRescanLastRun = app.querySelector('[data-scheduled-rescan-last-run]');
const scheduledRescanNextRun = app.querySelector('[data-scheduled-rescan-next-run]');
const scheduledRescanLastResult = app.querySelector('[data-scheduled-rescan-last-result]');
const cleanupHistorySummary = app.querySelector('[data-cleanup-history-summary]');
const cleanupHistoryContainer = app.querySelector('[data-cleanup-history]');
const clearCleanupHistoryButton = app.querySelector('[data-clear-cleanup-history]');
const restoreSummary = app.querySelector('[data-restore-summary]');
const restorePreviewContainer = app.querySelector('[data-restore-preview]');
const previewRestoreButton = app.querySelector('[data-preview-restore]');
const executeRestoreButton = app.querySelector('[data-execute-restore]');

const state = {
  discovery: null,
  progress: [],
  warnings: [],
  scanResult: null,
  expertMode: false,
  cleanupMode: 'review',
  cleanupBrowser: 'all',
  cleanupLockResolution: 'retryAfterManualClose',
  includeGeneralCleanup: false,
  aggressiveConfirmed: false,
  automaticCloseConfirmed: false,
  allowNoBackup: false,
  cleanupPreview: null,
  cleanupExecution: null,
  cleanupPreviewRunning: false,
  restorePreview: null,
  restoreExecution: null,
  restorePreviewRunning: false,
  cleanupHistory: [],
  scanRunning: false,
  scheduler: null,
  schedulerLoading: false,
  schedulerSaving: false,
  settings: {
    ruleBundleVersion: 'loading',
    updateState: 'loading',
    telemetryOptIn: false,
    diagnosticsOptIn: false,
  },
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
  if (state.cleanupPreviewRunning) {
    cleanupSummary.textContent = 'Generating cleanup preview...';
    cleanupPreviewContainer.innerHTML = `
      <div class="preview-loading" aria-live="polite" aria-busy="true">
        <div class="preview-loading__spinner" aria-hidden="true"></div>
        <div>
          <strong>Generating cleanup preview</strong>
          <p class="subtle">Building the cleanup plan from the latest scan. This can take a moment.</p>
        </div>
      </div>
    `;
    return;
  }

  if (!state.cleanupPreview) {
    cleanupSummary.textContent = 'No cleanup plan yet.';
    cleanupPreviewContainer.innerHTML =
      '<p class="empty">Run a scan, then preview a cleanup mode to review actions and locked-item choices.</p>';
    return;
  }

  const plan = state.cleanupPreview.plan;
  const lockedActionIds = state.cleanupPreview.lockedActionIds ?? state.cleanupPreview.locked_action_ids ?? [];
  const lockedProfiles = state.cleanupPreview.lockedProfiles ?? state.cleanupPreview.locked_profiles ?? [];
  const warnings = state.cleanupPreview.warnings ?? [];
  const requiresConfirmation =
    state.cleanupPreview.requiresConfirmation ?? state.cleanupPreview.requires_confirmation ?? false;
  const findings = cleanablePreviewFindings(state.scanResult);
  const trackerFindings = findings.filter((finding) => finding.trackerOwned);
  const generalFindings = findings.filter((finding) => !finding.trackerOwned);
  const selectedFindings = findings.filter((finding) => finding.trackerOwned || state.includeGeneralCleanup);
  const actionCount = plan.actions?.length ?? 0;
  const browserLabel = state.cleanupBrowser === 'all'
    ? 'all browsers'
    : state.cleanupBrowser;
  const lockLabel = {
    retryAfterManualClose: 'Retry after manual close',
    skipLocked: 'Skip locked actions',
    requestAutomaticClose: 'Request automatic close',
  }[state.cleanupLockResolution];
  cleanupSummary.textContent = `${cleanupScopeLabel(state.includeGeneralCleanup)} · ${plan.mode} mode · ${browserLabel} · ${selectedFindings.length} selected · ${trackerFindings.length} tracker item(s) · ${generalFindings.length} general item(s) · ${lockedActionIds.length} locked · ${lockLabel}`;

  cleanupPreviewContainer.innerHTML = `
    <div class="row">
      <strong>Warnings</strong>
      <div class="stack nested">
        ${warnings.length === 0 ? '<p class="empty">No cleanup warnings.</p>' : warnings.map((warning) => `<p class="row warning">${warning}</p>`).join('')}
      </div>
    </div>
    <div class="row">
      <strong>Tracker data</strong>
      <p class="subtle">Known tracker-owned cookies and storage are selected by default.</p>
      <div class="stack nested">
        ${trackerFindings.length === 0
          ? '<p class="empty">No tracker-owned cleanup candidates in this scan.</p>'
          : trackerFindings
              .map(
                (finding) => `
                  <div class="row artifact">
                    <div class="artifact-head">
                      <strong>${finding.artifactType}</strong>
                      <span>${finding.site ?? 'tracker-owned'}</span>
                      <span>${cleanupImpactLabel(finding.cleanupImpact)}</span>
                    </div>
                    <p class="artifact-detail">${finding.id}</p>
                    <p class="artifact-detail">${cleanupArtifactImpact(finding.artifactType, finding.cleanupImpact)}</p>
                  </div>
                `,
              )
              .join('')}
      </div>
    </div>
    <div class="row">
      <strong>General privacy cleanup</strong>
      <p class="subtle">${generalFindings.length === 0 ? 'No general browser data was identified for this scan.' : 'General cache, history, and ambiguous site data remain unselected unless you explicitly include them.'}</p>
      <div class="stack nested">
        ${generalFindings.length === 0
          ? '<p class="empty">No general browser items to review.</p>'
          : generalFindings
              .map(
                (finding) => `
                  <div class="row artifact">
                    <div class="artifact-head">
                      <strong>${finding.artifactType}</strong>
                      <span>${finding.site ?? 'browser data'}</span>
                      <span>${cleanupImpactLabel(finding.cleanupImpact)}</span>
                    </div>
                    <p class="artifact-detail">${finding.id}</p>
                    <p class="artifact-detail">${cleanupArtifactImpact(finding.artifactType, finding.cleanupImpact)}</p>
                  </div>
                `,
              )
              .join('')}
      </div>
    </div>
    <div class="row">
      <strong>Impact guide</strong>
      <p class="subtle">Cleanup impact depends on the artifact type. This guide explains the common effects before you continue.</p>
      <div class="stack nested">
        ${[
          ['cookie', 'may_sign_out'],
          ['local_storage', 'may_remove_preferences'],
          ['indexed_db', 'review_required'],
          ['cache', 'review_required'],
          ['service_worker', 'review_required'],
          ['history', 'review_required'],
        ]
          .map(([artifactType, cleanupImpact]) => `
            <div class="row artifact">
              <div class="artifact-head">
                <strong>${artifactType}</strong>
                <span>${cleanupImpactLabel(cleanupImpact)}</span>
              </div>
              <p class="artifact-detail">${cleanupArtifactImpact(artifactType, cleanupImpact)}</p>
            </div>
          `)
          .join('')}
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
      <strong>Locked profiles</strong>
      <div class="stack nested">
        ${lockedProfiles.length === 0
          ? '<p class="empty">No browser profiles are blocking cleanup.</p>'
          : lockedProfiles.map((profile) => `
              <div class="row artifact">
                <div class="artifact-head">
                  <strong>${profile.browser}</strong>
                  <span>${profile.profileName ?? profile.profile_name}</span>
                </div>
                <p class="artifact-detail">${profile.profilePath ?? profile.profile_path}</p>
              </div>
            `).join('')}
      </div>
    </div>
    <div class="row">
      <strong>Lock handling</strong>
      <p class="subtle">Selected action: ${lockLabel}.</p>
      ${state.cleanupLockResolution === 'requestAutomaticClose'
        ? `<p class="row warning">${autoCloseConfirm.checked ? 'Automatic browser closure will be requested on cleanup.' : 'Check confirmation to enable browser closure during cleanup.'}</p>`
        : ''}
      ${state.includeGeneralCleanup
        ? '<p class="row warning">General browser data is included in this cleanup request.</p>'
        : '<p class="subtle">General browser data remains excluded until you explicitly include it.</p>'}
    </div>
    <div class="row">
      <strong>Planned actions</strong>
      <div class="stack nested">
        ${actionCount === 0
          ? '<p class="empty">This mode selected no cleanup actions for the current scan.</p>'
          : plan.actions
              .map(
                (action) => `
                  <div class="row artifact">
                    <div class="artifact-head">
                      <strong>${action.id}</strong>
                      <span>${action.artifactType ?? action.artifact_type}</span>
                      <span>${action.requiresBrowserClosed ?? action.requires_browser_closed ? 'browser close required' : 'ready'}</span>
                    </div>
                    <p class="artifact-detail">${action.target.kind}</p>
                  </div>
                `,
              )
              .join('')}
      </div>
    </div>
    ${state.cleanupExecution ? `
    <div class="row">
      <strong>Cleanup result</strong>
      <div class="stack nested">
        <p class="row">Completed: ${(state.cleanupExecution.execution.completedIds ?? state.cleanupExecution.execution.completed_ids ?? []).length}</p>
        <p class="row">Skipped: ${(state.cleanupExecution.execution.skippedIds ?? state.cleanupExecution.execution.skipped_ids ?? []).length}</p>
        <p class="row">Failed: ${(state.cleanupExecution.execution.failed ?? []).length}</p>
        ${(state.cleanupExecution.execution.failed ?? []).length === 0
          ? ''
          : (state.cleanupExecution.execution.failed ?? [])
              .map((failure) => `<p class="row warning">${failure.id}: ${failure.message}</p>`)
              .join('')}
        ${state.cleanupExecution.verification ? `
          <div class="row">
            <strong>Verification</strong>
            <div class="stack nested">
              <p class="row">Removed: ${(state.cleanupExecution.verification.removedIds ?? state.cleanupExecution.verification.removed_ids ?? []).length}</p>
              <p class="row">Skipped: ${(state.cleanupExecution.verification.skippedIds ?? state.cleanupExecution.verification.skipped_ids ?? []).length}</p>
              <p class="row">Still detected: ${(state.cleanupExecution.verification.stillDetectedIds ?? state.cleanupExecution.verification.still_detected_ids ?? []).length}</p>
              <p class="row">Failed: ${(state.cleanupExecution.verification.failedIds ?? state.cleanupExecution.verification.failed_ids ?? []).length}</p>
              ${(state.cleanupExecution.verification.warnings ?? []).length === 0
                ? ''
                : (state.cleanupExecution.verification.warnings ?? [])
                    .map((warning) => `<p class="row warning">${warning}</p>`)
                    .join('')}
              ${((state.cleanupExecution.verification.stillDetectedIds ?? state.cleanupExecution.verification.still_detected_ids ?? []).length === 0
                ? ''
                : (state.cleanupExecution.verification.stillDetectedIds ?? state.cleanupExecution.verification.still_detected_ids ?? [])
                    .map((id) => `<p class="row warning">Still detected: ${id}</p>`)
                    .join(''))}
            </div>
          </div>
        ` : ''}
      </div>
    </div>
    ` : ''}
  `;
}

function renderSettings() {
  settingsSummary.textContent =
    `${state.settings.ruleBundleVersion} · ${state.settings.updateState}`;
  ruleVersion.textContent = state.settings.ruleBundleVersion;
  updateState.textContent = state.settings.updateState;
  telemetryOptIn.checked = state.settings.telemetryOptIn;
  diagnosticsOptIn.checked = state.settings.diagnosticsOptIn;
}

function formatSchedulerTimestamp(timestampMs) {
  if (!timestampMs) {
    return 'Never';
  }
  return new Date(timestampMs).toLocaleString();
}

function formatSchedulerTaskResult(result) {
  if (!result) {
    return 'Never run';
  }

  if (typeof result === 'string') {
    return result;
  }

  switch (result.kind) {
    case 'never_run':
      return 'Never run';
    case 'succeeded':
      return `Succeeded: ${result.message}`;
    case 'failed':
      return `Failed: ${result.message}`;
    default:
      return 'Unknown result';
  }
}

function schedulerTask(snapshot) {
  return snapshot ?? {
    enabled: false,
    intervalDays: 7,
    lastRunAtMs: null,
    nextRunAtMs: null,
    lastResult: { kind: 'never_run' },
  };
}

function renderScheduler() {
  const snapshot = state.scheduler;
  if (!snapshot) {
    schedulerSummary.textContent = 'Loading scheduled maintenance preferences...';
    schedulerPanel.classList.add('dimmed');
    ruleRefreshEnabled.checked = false;
    ruleRefreshFrequency.value = '7';
    ruleRefreshLastRun.textContent = 'Last run: Loading...';
    ruleRefreshNextRun.textContent = 'Next run: Loading...';
    ruleRefreshLastResult.textContent = 'Last result: Loading...';
    scheduledRescanEnabled.checked = false;
    scheduledRescanFrequency.value = '7';
    scheduledRescanLastRun.textContent = 'Last run: Loading...';
    scheduledRescanNextRun.textContent = 'Next run: Loading...';
    scheduledRescanLastResult.textContent = 'Last result: Loading...';
    return;
  }

  const refresh = schedulerTask(snapshot.ruleRefresh ?? snapshot.rule_refresh);
  const rescan = schedulerTask(snapshot.rescan);

  schedulerSummary.textContent = refresh.enabled || rescan.enabled
    ? 'Scheduled maintenance is enabled.'
    : 'Scheduled maintenance is disabled by default.';
  schedulerPanel.classList.remove('dimmed');

  ruleRefreshEnabled.checked = refresh.enabled;
  ruleRefreshFrequency.value = String(refresh.intervalDays ?? refresh.interval_days ?? 7);
  ruleRefreshLastRun.textContent = `Last run: ${formatSchedulerTimestamp(refresh.lastRunAtMs ?? refresh.last_run_at_ms)}`;
  ruleRefreshNextRun.textContent = refresh.enabled
    ? `Next run: ${formatSchedulerTimestamp(refresh.nextRunAtMs ?? refresh.next_run_at_ms)}`
    : 'Next run: Disabled';
  ruleRefreshLastResult.textContent = `Last result: ${formatSchedulerTaskResult(refresh.lastResult ?? refresh.last_result)}`;

  scheduledRescanEnabled.checked = rescan.enabled;
  scheduledRescanFrequency.value = String(rescan.intervalDays ?? rescan.interval_days ?? 7);
  scheduledRescanLastRun.textContent = `Last run: ${formatSchedulerTimestamp(rescan.lastRunAtMs ?? rescan.last_run_at_ms)}`;
  scheduledRescanNextRun.textContent = rescan.enabled
    ? `Next run: ${formatSchedulerTimestamp(rescan.nextRunAtMs ?? rescan.next_run_at_ms)}`
    : 'Next run: Disabled';
  scheduledRescanLastResult.textContent = `Last result: ${formatSchedulerTaskResult(rescan.lastResult ?? rescan.last_result)}`;
}

function syncSchedulerControls() {
  const disabled = state.schedulerLoading || state.schedulerSaving;
  ruleRefreshEnabled.disabled = disabled;
  ruleRefreshFrequency.disabled = disabled;
  scheduledRescanEnabled.disabled = disabled;
  scheduledRescanFrequency.disabled = disabled;
}

function schedulerUpdateRequest() {
  return {
    ruleRefreshEnabled: ruleRefreshEnabled.checked,
    ruleRefreshIntervalDays: Number(ruleRefreshFrequency.value),
    rescanEnabled: scheduledRescanEnabled.checked,
    rescanIntervalDays: Number(scheduledRescanFrequency.value),
  };
}

function renderCleanupHistory() {
  const records = state.cleanupHistory ?? [];
  cleanupHistorySummary.textContent =
    records.length === 0 ? 'No cleanup history yet.' : `${records.length} record(s) stored locally.`;
  clearCleanupHistoryButton.disabled = records.length === 0;

  if (records.length === 0) {
    cleanupHistoryContainer.innerHTML = '<p class="empty">Run a cleanup to record local audit history.</p>';
    return;
  }

  cleanupHistoryContainer.innerHTML = records
    .map(
      (record) => `
        <div class="row artifact">
          <div class="artifact-head">
            <strong>${new Date(record.timestampMs).toLocaleString()}</strong>
            <span>${record.browser}</span>
            <span>${record.profileName}</span>
            <span>${record.mode}</span>
          </div>
          <p class="artifact-detail">${record.actionId} · ${record.artifactType} · ${formatCleanupAuditOutcome(record.outcome)}</p>
          <p class="artifact-detail">Bundle ${record.ruleBundleVersion} · ${record.profilePath}</p>
        </div>
      `,
    )
    .join('');
}

function renderRestorePreview() {
  const preview = state.restorePreview;
  const records = preview?.records ?? [];
  const warnings = preview?.warnings ?? [];
  const execution = state.restoreExecution;

  if (!preview) {
    if (!execution) {
      restoreSummary.textContent = 'No restore preview yet.';
      restorePreviewContainer.innerHTML = '<p class="empty">Preview a restore to see what will be recovered.</p>';
    } else {
      const completed = execution.completedIds ?? execution.completed_ids ?? [];
      const skipped = execution.skippedIds ?? execution.skipped_ids ?? [];
      const failed = execution.failed ?? [];
      restoreSummary.textContent =
        failed.length > 0
          ? `Restore finished with ${failed.length} failure(s).`
          : skipped.length > 0
            ? `Restore finished with ${skipped.length} skipped item(s).`
            : `Restore finished successfully for ${completed.length} item(s).`;
      restorePreviewContainer.innerHTML = `
        <div class="row">
          <strong>Restore result</strong>
          <div class="stack nested">
            <p class="row">Completed: ${completed.length}</p>
            <p class="row">Skipped: ${skipped.length}</p>
            <p class="row">Failed: ${failed.length}</p>
            ${failed.length === 0
              ? ''
              : failed
                  .map((failure) => `<p class="row warning">${failure.actionId ?? failure.action_id}: ${failure.message}</p>`)
                  .join('')}
          </div>
        </div>
      `;
    }
    executeRestoreButton.disabled = true;
    return;
  }

  if (records.length === 0) {
    restoreSummary.textContent = warnings[0] ?? 'No restore candidates available.';
    restorePreviewContainer.innerHTML = warnings.length === 0
      ? '<p class="empty">No cleanup backups are available for restore.</p>'
      : warnings.map((warning) => `<p class="row warning">${warning}</p>`).join('');
    executeRestoreButton.disabled = true;
    return;
  }

  restoreSummary.textContent = `${records.length} backup item(s) ready to restore.`;
  executeRestoreButton.disabled = state.scanRunning || state.restorePreviewRunning;

  restorePreviewContainer.innerHTML = `
    ${warnings.length === 0
      ? ''
      : warnings.map((warning) => `<p class="row warning">${warning}</p>`).join('')}
    ${records
      .map(
        (record) => `
          <div class="row artifact">
            <div class="artifact-head">
              <strong>${new Date(record.timestampMs).toLocaleString()}</strong>
              <span>${record.browser}</span>
              <span>${record.profileName}</span>
              <span>${record.artifactType}</span>
            </div>
            <p class="artifact-detail">${record.actionId} · ${record.mode} · bundle ${record.ruleBundleVersion}</p>
            <p class="artifact-detail">${record.profilePath} · backup ${record.backupPath}</p>
          </div>
        `,
      )
      .join('')}
    ${execution ? `
      <div class="row">
        <strong>Restore result</strong>
        <div class="stack nested">
          <p class="row">Completed: ${execution.completedIds?.length ?? execution.completed_ids?.length ?? 0}</p>
          <p class="row">Skipped: ${execution.skippedIds?.length ?? execution.skipped_ids?.length ?? 0}</p>
          <p class="row">Failed: ${(execution.failed ?? []).length}</p>
          ${(execution.failed ?? []).length === 0
            ? ''
            : (execution.failed ?? [])
                .map((failure) => `<p class="row warning">${failure.actionId ?? failure.action_id}: ${failure.message}</p>`)
                .join('')}
        </div>
      </div>
    ` : ''}
  `;
}

function syncCleanupControls() {
  aggressiveConfirm.disabled = state.scanRunning || state.cleanupMode !== 'aggressive';
  autoCloseConfirm.disabled = state.scanRunning || state.cleanupLockResolution !== 'requestAutomaticClose';
  const generalFindingCount = cleanablePreviewFindings(state.scanResult).filter(
    (finding) => !finding.trackerOwned,
  ).length;
  includeGeneralCleanup.disabled = state.scanRunning || !state.scanResult || generalFindingCount === 0;
  includeGeneralCleanup.checked = state.includeGeneralCleanup && !includeGeneralCleanup.disabled;
  state.includeGeneralCleanup = includeGeneralCleanup.checked;
  previewCleanupButton.textContent = state.cleanupPreviewRunning
    ? 'Generating...'
    : 'Preview cleanup';
  previewCleanupButton.disabled =
    state.scanRunning ||
    state.cleanupPreviewRunning ||
    !state.scanResult ||
    (state.cleanupMode === 'aggressive' && !aggressiveConfirm.checked);
  executeCleanupButton.disabled =
    state.scanRunning ||
    state.cleanupPreviewRunning ||
    !state.cleanupPreview ||
    (state.cleanupMode === 'aggressive' && !aggressiveConfirm.checked) ||
    (state.cleanupLockResolution === 'requestAutomaticClose' && !autoCloseConfirm.checked);
  executeCleanupButton.textContent = state.cleanupLockResolution === 'requestAutomaticClose'
    ? 'Close browsers and clean'
    : 'Clean trackers';
}

function syncRestoreControls() {
  previewRestoreButton.disabled = state.scanRunning || state.restorePreviewRunning;
  previewRestoreButton.textContent = state.restorePreviewRunning ? 'Generating...' : 'Preview restore';
  executeRestoreButton.disabled =
    state.scanRunning ||
    state.restorePreviewRunning ||
    !state.restorePreview ||
    (state.restorePreview.records ?? []).length === 0;
  executeRestoreButton.textContent = 'Restore latest cleanup';
}

function setRunning(running) {
  state.scanRunning = running;
  startButton.disabled = running || !state.discovery;
  cancelButton.disabled = !running;
  syncCleanupControls();
  syncRestoreControls();
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

function formatUpdateState(value) {
  switch (value) {
    case 'embedded_starter_bundle':
      return 'Embedded starter bundle';
    default:
      return value.replaceAll('_', ' ');
  }
}

function readStoredFlag(key) {
  try {
    return localStorage.getItem(key) === 'true';
  } catch {
    return false;
  }
}

function writeStoredFlag(key, value) {
  try {
    localStorage.setItem(key, value ? 'true' : 'false');
  } catch {
    // Ignore storage failures in restricted or private contexts.
  }
}

function cleanableFindingIds(scanResult) {
  return cleanablePreviewFindings(scanResult).map((finding) => finding.id);
}

function formatCleanupAuditOutcome(outcome) {
  if (!outcome) {
    return 'unknown';
  }

  if (typeof outcome === 'string') {
    return outcome.replaceAll('_', ' ');
  }

  switch (outcome.kind) {
    case 'completed':
      return 'completed';
    case 'skipped':
      return 'skipped';
    case 'failed':
      return `failed: ${outcome.message}`;
    case 'blocked':
      return `blocked: ${outcome.reason}`;
    default:
      return 'unknown';
  }
}

function isTrackerOwnedFinding(finding) {
  return finding.classification?.ownership === 'tracker_owned';
}

function cleanablePreviewFindings(scanResult) {
  const cleanableArtifactTypes = new Set([
    'cookie',
    'local_storage',
    'indexed_db',
    'cache',
    'history',
    'service_worker',
  ]);

  return (scanResult?.profiles ?? [])
    .filter((profile) =>
      state.cleanupBrowser === 'all' ||
      profile.browser.toLowerCase() === state.cleanupBrowser,
    )
    .flatMap((profile) =>
      profile.findings
      .filter((finding) =>
        cleanableArtifactTypes.has(finding.artifact_type) &&
        (finding.artifact_type !== 'cookie' || Boolean(finding.site)),
      )
      .map((finding) => ({
        id: finding.id,
        artifactType: finding.artifact_type,
        site: finding.site,
        confidence: finding.confidence,
        cleanupImpact: finding.cleanup_impact,
        trackerOwned: isTrackerOwnedFinding(finding),
      })),
    );
}

function selectedCleanupFindingIds(scanResult) {
  return cleanablePreviewFindings(scanResult)
    .filter((finding) => finding.trackerOwned || state.includeGeneralCleanup)
    .map((finding) => finding.id);
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
    state.includeGeneralCleanup = false;
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
  const snapshot = await invoke('discover_profiles', {
    request: {},
  });
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

async function loadSettings() {
  const snapshot = await invoke('settings_snapshot', {});
  state.settings.ruleBundleVersion = snapshot.ruleBundleVersion;
  state.settings.updateState = formatUpdateState(snapshot.updateState);
  state.settings.telemetryOptIn = readStoredFlag('trackers.telemetryOptIn');
  state.settings.diagnosticsOptIn = readStoredFlag('trackers.diagnosticsOptIn');
  renderSettings();
}

async function loadScheduler() {
  state.schedulerLoading = true;
  syncSchedulerControls();
  try {
    state.scheduler = await invoke('scheduler_snapshot', {});
    renderScheduler();
  } finally {
    state.schedulerLoading = false;
    syncSchedulerControls();
    renderScheduler();
  }
}

async function saveScheduler() {
  if (!state.scheduler) {
    return;
  }

  state.schedulerSaving = true;
  syncSchedulerControls();
  renderScheduler();

  try {
    state.scheduler = await invoke('update_scheduler_settings', {
      request: schedulerUpdateRequest(),
    });
    renderScheduler();
    setStatus('Scheduled maintenance preferences saved.');
  } catch (error) {
    setStatus(`Scheduled maintenance update failed: ${error}`);
    await loadScheduler();
  } finally {
    state.schedulerSaving = false;
    syncSchedulerControls();
    renderScheduler();
  }
}

async function loadCleanupHistory() {
  const history = await invoke('cleanup_audit_history', {});
  state.cleanupHistory = history.records ?? [];
  renderCleanupHistory();
}

async function startScan() {
  if (!state.discovery || state.scanRunning) {
    return;
  }

  state.progress = [];
  state.scanResult = null;
  state.cleanupPreview = null;
  state.cleanupExecution = null;
  state.includeGeneralCleanup = false;
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

  state.cleanupPreviewRunning = true;
  syncCleanupControls();
  setStatus('Generating cleanup preview...');

  try {
    const preview = await invoke('preview_cleanup', {
      request: {
        mode: state.cleanupMode,
        selectedFindingIds: selectedCleanupFindingIds(state.scanResult),
        aggressiveConfirmed: state.cleanupMode !== 'aggressive' || aggressiveConfirm.checked,
      },
    });

    state.cleanupPreview = preview;
    state.cleanupExecution = null;
    renderCleanupPreview();
    setStatus(`Cleanup preview ready in ${state.cleanupMode} mode.`);
  } finally {
    state.cleanupPreviewRunning = false;
    syncCleanupControls();
    renderCleanupPreview();
  }
}

async function previewRestore() {
  state.restorePreviewRunning = true;
  syncRestoreControls();
  setStatus('Generating restore preview...');

  try {
    const preview = await invoke('restore_cleanup_preview', {});
    state.restorePreview = preview;
    state.restoreExecution = null;
    renderRestorePreview();
    setStatus('Restore preview ready.');
  } finally {
    state.restorePreviewRunning = false;
    syncRestoreControls();
    renderRestorePreview();
  }
}

async function executeRestore() {
  if (!state.restorePreview || state.scanRunning) {
    return;
  }

  const result = await invoke('restore_cleanup', {
    request: state.restorePreview,
  });

  state.restoreExecution = result;
  const completed = result.completedIds ?? result.completed_ids ?? [];
  const skipped = result.skippedIds ?? result.skipped_ids ?? [];
  const failed = result.failed ?? [];

  if (failed.length === 0 && skipped.length === 0) {
    state.restorePreview = null;
  }
  renderRestorePreview();
  syncRestoreControls();

  if (failed.length > 0) {
    setStatus(`Restore finished with ${failed.length} failure(s).`);
    return;
  }
  if (skipped.length > 0) {
    setStatus(`Restore finished with ${skipped.length} skipped item(s).`);
    return;
  }
  setStatus(`Restore finished successfully for ${completed.length} item(s).`);
}

async function executeCleanup() {
  if (!state.cleanupPreview || state.scanRunning) {
    return;
  }

  const result = await invoke('execute_cleanup', {
    request: {
      preview: state.cleanupPreview,
      lockResolution: (() => {
        if (state.cleanupLockResolution === 'requestAutomaticClose') {
          return { requestAutomaticClose: { confirmed: autoCloseConfirm.checked } };
        }
        if (state.cleanupLockResolution === 'skipLocked') {
          return 'skipLocked';
        }
        return 'retryAfterManualClose';
      })(),
      allowNoBackup: allowNoBackup.checked,
    },
  });

  state.cleanupExecution = result;
  renderCleanupPreview();
  loadCleanupHistory().catch((error) => {
    setStatus(`Cleanup history refresh failed: ${error}`);
  });

  const status = result.status ?? {};
  if (status.kind === 'backup_failed') {
    setStatus(`Cleanup failed: backup failed: ${status.message}`);
    return;
  }
  if (status.kind === 'retry_after_close') {
    setStatus('Cleanup paused: close the locked browsers and try again.');
    return;
  }
  if (status.kind === 'confirmation_required') {
    setStatus('Cleanup paused: confirm automatic browser close to continue.');
    return;
  }
  if (status.kind === 'browser_close_failed') {
    setStatus(`Cleanup failed: ${status.message}`);
    return;
  }

  const execution = result.execution;
  const skippedIds = execution.skippedIds ?? execution.skipped_ids ?? [];
  const verification = result.verification ?? {};
  const stillDetectedIds = verification.stillDetectedIds ?? verification.still_detected_ids ?? [];
  if (execution.failed.length > 0) {
    setStatus(`Cleanup finished with ${execution.failed.length} failure(s).`);
  } else if (stillDetectedIds.length > 0) {
    setStatus(`Cleanup finished, but ${stillDetectedIds.length} tracker artifact(s) remain detected.`);
  } else if (skippedIds.length > 0) {
    setStatus(`Cleanup finished with ${skippedIds.length} skipped action(s).`);
  } else {
    setStatus('Cleanup finished successfully.');
  }

  state.restorePreview = null;
  state.restoreExecution = null;
  renderRestorePreview();
  syncRestoreControls();
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
  state.cleanupExecution = null;
  renderCleanupPreview();
  syncCleanupControls();
}

function setCleanupBrowser(browser) {
  state.cleanupBrowser = browser;
  state.cleanupPreview = null;
  state.cleanupExecution = null;
  renderCleanupPreview();
  syncCleanupControls();
}

function setTelemetryOptIn(enabled) {
  state.settings.telemetryOptIn = enabled;
  writeStoredFlag('trackers.telemetryOptIn', enabled);
  renderSettings();
}

function setDiagnosticsOptIn(enabled) {
  state.settings.diagnosticsOptIn = enabled;
  writeStoredFlag('trackers.diagnosticsOptIn', enabled);
  renderSettings();
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

previewRestoreButton.addEventListener('click', () => {
  previewRestore().catch((error) => {
    setStatus(`Restore preview failed: ${error}`);
  });
});

executeCleanupButton.addEventListener('click', () => {
  executeCleanup().catch((error) => {
    setStatus(`Cleanup failed: ${error}`);
  });
});

executeRestoreButton.addEventListener('click', () => {
  executeRestore().catch((error) => {
    setStatus(`Restore failed: ${error}`);
  });
});

cleanupModeButtons.forEach((button) => {
  button.addEventListener('click', () => {
    setCleanupMode(button.dataset.cleanupMode);
  });
});

aggressiveConfirm.addEventListener('change', () => {
  state.aggressiveConfirmed = aggressiveConfirm.checked;
  state.cleanupPreview = null;
  renderCleanupPreview();
  syncCleanupControls();
});

cleanupBrowser.addEventListener('change', () => {
  setCleanupBrowser(cleanupBrowser.value);
});

includeGeneralCleanup.addEventListener('change', () => {
  state.includeGeneralCleanup = includeGeneralCleanup.checked;
  state.cleanupPreview = null;
  renderCleanupPreview();
  syncCleanupControls();
});

lockResolution.addEventListener('change', () => {
  state.cleanupLockResolution = lockResolution.value;
  if (state.cleanupLockResolution !== 'requestAutomaticClose') {
    autoCloseConfirm.checked = false;
  }
  state.cleanupPreview = null;
  state.cleanupExecution = null;
  renderCleanupPreview();
  syncCleanupControls();
});

autoCloseConfirm.addEventListener('change', () => {
  state.automaticCloseConfirmed = autoCloseConfirm.checked;
  syncCleanupControls();
  renderCleanupPreview();
});

telemetryOptIn.addEventListener('change', () => {
  setTelemetryOptIn(telemetryOptIn.checked);
});

diagnosticsOptIn.addEventListener('change', () => {
  setDiagnosticsOptIn(diagnosticsOptIn.checked);
});

ruleRefreshEnabled.addEventListener('change', () => {
  if (state.scheduler) {
    state.scheduler.ruleRefresh.enabled = ruleRefreshEnabled.checked;
    saveScheduler().catch((error) => {
      setStatus(`Scheduled maintenance update failed: ${error}`);
    });
  }
});

ruleRefreshFrequency.addEventListener('change', () => {
  if (state.scheduler) {
    state.scheduler.ruleRefresh.intervalDays = Number(ruleRefreshFrequency.value);
    saveScheduler().catch((error) => {
      setStatus(`Scheduled maintenance update failed: ${error}`);
    });
  }
});

scheduledRescanEnabled.addEventListener('change', () => {
  if (state.scheduler) {
    state.scheduler.rescan.enabled = scheduledRescanEnabled.checked;
    saveScheduler().catch((error) => {
      setStatus(`Scheduled maintenance update failed: ${error}`);
    });
  }
});

scheduledRescanFrequency.addEventListener('change', () => {
  if (state.scheduler) {
    state.scheduler.rescan.intervalDays = Number(scheduledRescanFrequency.value);
    saveScheduler().catch((error) => {
      setStatus(`Scheduled maintenance update failed: ${error}`);
    });
  }
});

clearCleanupHistoryButton.addEventListener('click', () => {
  invoke('clear_cleanup_audit_history', {})
    .then(() => loadCleanupHistory())
    .then(() => {
      setStatus('Cleanup history cleared.');
    })
    .catch((error) => {
      setStatus(`Clear cleanup history failed: ${error}`);
    });
});

registerListeners();
setCleanupMode('review');
lockResolution.value = state.cleanupLockResolution;
renderSettings();
renderScheduler();
renderRestorePreview();
syncRestoreControls();
loadDiscovery().catch((error) => {
  setStatus(`Failed to load profiles: ${error}`);
});
loadSettings().catch((error) => {
  state.settings.ruleBundleVersion = 'unavailable';
  state.settings.updateState = `Unavailable: ${error}`;
  renderSettings();
});
loadScheduler().catch((error) => {
  schedulerSummary.textContent = `Scheduled maintenance unavailable: ${error}`;
  schedulerPanel.innerHTML = '<p class="empty">Unable to load scheduled maintenance preferences.</p>';
});
loadCleanupHistory().catch((error) => {
  cleanupHistorySummary.textContent = `Cleanup history unavailable: ${error}`;
  cleanupHistoryContainer.innerHTML = '<p class="empty">Unable to load cleanup history.</p>';
});
