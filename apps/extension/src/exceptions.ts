export interface SiteException {
  site: string;
  expiresAt?: number;
}

export function pauseSitePermanently(
  exceptions: SiteException[],
  site: string,
): SiteException[] {
  return upsert(exceptions, { site });
}

export function pauseSiteTemporarily(
  exceptions: SiteException[],
  site: string,
  now: number,
  durationMs: number,
): SiteException[] {
  return upsert(exceptions, { site, expiresAt: now + durationMs });
}

export function isSitePaused(
  exceptions: SiteException[],
  site: string,
  now: number,
): boolean {
  return exceptions.some(
    (exception) =>
      exception.site === site &&
      (exception.expiresAt === undefined || exception.expiresAt > now),
  );
}

function upsert(
  exceptions: SiteException[],
  replacement: SiteException,
): SiteException[] {
  return [
    ...exceptions.filter(({ site }) => site !== replacement.site),
    replacement,
  ];
}
