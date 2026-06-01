export function buildLayeredResults(scanResult, options = {}) {
  const expertMode = options.expertMode === true;
  const browserGroups = new Map();

  for (const profileResult of scanResult.profiles ?? []) {
    const browserKey = profileResult.browser;
    const profileKey = profileResult.profileName;
    const browserGroup = browserGroups.get(browserKey) ?? {
      browser: browserKey,
      profiles: new Map(),
    };
    browserGroups.set(browserKey, browserGroup);

    const profileGroup = browserGroup.profiles.get(profileKey) ?? {
      profileName: profileResult.profileName,
      profilePath: profileResult.profilePath,
      warnings: profileResult.warnings ?? [],
      groups: new Map(),
    };
    browserGroup.profiles.set(profileKey, profileGroup);

    for (const finding of profileResult.findings ?? []) {
      const siteKey = finding.site ?? '(no site)';
      const siteGroup = profileGroup.groups.get(siteKey) ?? {
        site: finding.site,
        artifacts: new Map(),
      };
      profileGroup.groups.set(siteKey, siteGroup);

      const artifactKey = `${finding.artifact_type}|${finding.confidence ?? 'none'}`;
      const artifactGroup = siteGroup.artifacts.get(artifactKey) ?? {
        artifactType: finding.artifact_type,
        confidence: finding.confidence,
        cleanupImpact: finding.cleanup_impact,
        findings: [],
      };

      const detail = {
        id: finding.id,
        artifactType: finding.artifact_type,
        confidence: finding.confidence,
        cleanupImpact: finding.cleanup_impact,
      };

      if (expertMode) {
        detail.evidenceSummary = finding.evidence_summary;
        detail.profilePath = finding.profile?.profile_path;
        artifactGroup.evidenceSummary = detail.evidenceSummary;
        artifactGroup.profilePath = detail.profilePath;
      }

      artifactGroup.findings.push(detail);
      siteGroup.artifacts.set(artifactKey, artifactGroup);
    }
  }

  return {
    browsers: [...browserGroups.values()]
      .sort((left, right) => left.browser.localeCompare(right.browser))
      .map((browserGroup) => ({
        browser: browserGroup.browser,
        profiles: [...browserGroup.profiles.values()]
          .sort((left, right) => left.profileName.localeCompare(right.profileName))
          .map((profileGroup) => ({
            profileName: profileGroup.profileName,
            profilePath: profileGroup.profilePath,
            warnings: profileGroup.warnings,
            groups: [...profileGroup.groups.values()]
              .sort((left, right) =>
                (left.site ?? '').localeCompare(right.site ?? ''),
              )
              .map((siteGroup) => ({
                site: siteGroup.site,
                artifacts: [...siteGroup.artifacts.values()].sort((left, right) =>
                  left.artifactType.localeCompare(right.artifactType),
                ),
              })),
          })),
      })),
  };
}
