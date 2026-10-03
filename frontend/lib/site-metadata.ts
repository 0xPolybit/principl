import type { Metadata } from "next";
import { canonicalUrl } from "@/lib/site-url";

export function publicPageMetadata(title: string, description: string, path: string): Metadata {
  const canonical = canonicalUrl(path);
  return {
    title: { absolute: title },
    description,
    ...(canonical ? { alternates: { canonical } } : {}),
    openGraph: {
      type: "website",
      siteName: "PrinciPL",
      title,
      description,
      ...(canonical ? { url: canonical } : {}),
    },
    twitter: { card: "summary_large_image", title, description },
  };
}
