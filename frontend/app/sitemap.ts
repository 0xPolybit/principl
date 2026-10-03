import type { MetadataRoute } from "next";
import { docsPages, docsHref } from "@/content/docs/navigation";
import { getPublicSiteOrigin } from "@/lib/site-url";

const publicPages = ["/", "/docs", "/examples"];

export default function sitemap(): MetadataRoute.Sitemap {
  const origin = getPublicSiteOrigin();
  if (!origin) return [];

  const paths = [...publicPages, ...docsPages.map(docsHref)];
  return [...new Set(paths)].map((path) => ({
    url: new URL(path, origin).toString(),
  }));
}
