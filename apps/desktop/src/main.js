import { invoke } from '@tauri-apps/api/core';
import './style.css';

const app = document.querySelector('#app');

app.innerHTML = `
  <main class="shell">
    <section class="hero">
      <p class="eyebrow">Desktop shell</p>
      <h1>Trackers</h1>
      <p class="lede">
        Local browser discovery snapshot for Chrome and Edge.
      </p>
    </section>

    <section class="panel" aria-live="polite">
      <p class="status" data-status>Loading scanner core...</p>
      <div class="grid" data-results hidden></div>
    </section>
  </main>
`;

const status = app.querySelector('[data-status]');
const results = app.querySelector('[data-results]');

function renderBrowserCard(label, result) {
  const card = document.createElement('article');
  card.className = 'card';
  card.innerHTML = `
    <h2>${label}</h2>
    <p>${result.profiles.length} profile(s) found</p>
    <p>${result.warnings.length} warning(s)</p>
  `;
  return card;
}

async function bootstrap() {
  const snapshot = await invoke('discover_profiles', {});
  status.textContent = 'Scanner core connected.';
  results.hidden = false;
  results.replaceChildren(
    renderBrowserCard('Chrome', snapshot.chrome),
    renderBrowserCard('Edge', snapshot.edge),
  );
}

bootstrap().catch((error) => {
  status.textContent = `Desktop shell failed to bootstrap: ${error}`;
});
