const ARTIFACT_IMPACT_TEXT = {
  cookie: {
    may_sign_out: 'Cookies can sign you out of websites and reset login sessions.',
    may_remove_preferences:
      'Cookies can store small site settings and login helpers; removing them can reset session state.',
    review_required: 'Cookies can remove session data and may sign you out of websites.',
    low: 'Cookies are usually low impact when isolated to a known tracker host.',
  },
  local_storage: {
    may_sign_out: 'Local Storage can hold login helpers that some sites use to keep sessions active.',
    may_remove_preferences:
      'Local Storage can remove saved preferences, settings, and site state.',
    review_required:
      'Local Storage can reset site state and remove data a page expects to find on reload.',
    low: 'Local Storage cleanup usually removes site state without changing sign-in status.',
  },
  indexed_db: {
    may_sign_out:
      'IndexedDB can remove stored app state and may break logins for apps that depend on local data.',
    may_remove_preferences:
      'IndexedDB can remove offline app data and stored records that websites use to resume state.',
    review_required:
      'IndexedDB can remove offline app data and stored records that websites use to resume state.',
    low: 'IndexedDB cleanup may remove local app data used by sites to resume state.',
  },
  cache: {
    may_sign_out: 'Cache cleanup usually does not sign you out, but it can remove cached assets.',
    may_remove_preferences: 'Cache cleanup can force slower reloads and re-download site assets.',
    review_required: 'Cache cleanup can force slower reloads and re-download site assets.',
    low: 'Cache cleanup can clear stored assets and make pages load from the network again.',
  },
  service_worker: {
    may_sign_out:
      'Service workers can affect offline support, background sync, and push notifications.',
    may_remove_preferences:
      'Service workers can affect offline support, background sync, and push notifications.',
    review_required:
      'Service workers can affect offline support, background sync, and push notifications.',
    low: 'Service workers can affect offline support and background notifications.',
  },
  history: {
    may_sign_out: 'History cleanup removes browsing history and may clear saved visit traces.',
    may_remove_preferences:
      'History cleanup removes browsing history, search suggestions, and visit traces.',
    review_required: 'History cleanup removes browsing history, search suggestions, and visit traces.',
    low: 'History cleanup removes visit traces and can reduce suggestions in the browser UI.',
  },
};

const CLEANUP_IMPACT_LABELS = {
  low: 'Low impact',
  may_remove_preferences: 'Preference loss',
  may_sign_out: 'Sign-out risk',
  review_required: 'Review required',
};

const CLEANUP_SCOPE_LABELS = {
  false: 'Tracker cleanup',
  true: 'General privacy cleanup',
};

export function cleanupArtifactImpact(artifactType, cleanupImpact) {
  const impactKey = cleanupImpact ?? 'review_required';
  return (
    ARTIFACT_IMPACT_TEXT[artifactType]?.[impactKey] ??
    'Cleanup impact depends on the site and may remove local browser state.'
  );
}

export function cleanupImpactLabel(cleanupImpact) {
  return CLEANUP_IMPACT_LABELS[cleanupImpact] ?? 'Review required';
}

export function cleanupScopeLabel(includeGeneralCleanup) {
  return CLEANUP_SCOPE_LABELS[String(includeGeneralCleanup)] ?? 'Tracker cleanup';
}
