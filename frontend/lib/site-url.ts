/**
 * The project has no canonical public hostname configured in the repository.
 * Deployments can opt in through NEXT_PUBLIC_SITE_URL; no placeholder host is
 * emitted into canonical tags, sitemaps, robots metadata, or structured data.
 */
export function getPublicSiteOrigin(): string | undefined {
  const configured = process.env.NEXT_PUBLIC_SITE_URL?.trim();
  if (!configured) return undefined;

  try {
    const url = new URL(configured);
    if (url.protocol !== "https:" && url.protocol !== "http:") return undefined;
    return url.origin;
  } catch {
    return undefined;
  }
}

export function getMetadataBaseUrl(): URL {
  const publicOrigin = getPublicSiteOrigin();
  if (publicOrigin) return new URL(publicOrigin);

  const deploymentHost =
    process.env.VERCEL_PROJECT_PRODUCTION_URL ?? process.env.VERCEL_URL;
  if (deploymentHost) {
    return new URL(deploymentHost.startsWith("http") ? deploymentHost : `https://${deploymentHost}`);
  }

  // The local origin is only a metadata resolution fallback. Canonical URLs,
  // robots sitemap links, and sitemap entries remain disabled until configured.
  return new URL("http://localhost:3000");
}

export function canonicalUrl(path: string): string | undefined {
  const origin = getPublicSiteOrigin();
  return origin ? new URL(path, origin).toString() : undefined;
}
