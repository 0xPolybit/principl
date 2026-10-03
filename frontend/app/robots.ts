import type { MetadataRoute } from "next";
import { getPublicSiteOrigin } from "@/lib/site-url";

export default function robots(): MetadataRoute.Robots {
  const origin = getPublicSiteOrigin();
  return {
    rules: { userAgent: "*", allow: "/", disallow: "/docs/search-index" },
    ...(origin ? { sitemap: `${origin}/sitemap.xml` } : {}),
  };
}
