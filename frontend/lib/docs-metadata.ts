import type { Metadata } from "next";
import { canonicalUrl } from "@/lib/site-url";

export function docsMetadata(title: string, description: string, path: string): Metadata {
  const absoluteTitle = `${title} — PrinciPL Documentation`;
  const canonical = canonicalUrl(path);

  return {
    title: { absolute: absoluteTitle },
    description,
    ...(canonical ? { alternates: { canonical } } : {}),
    openGraph: {
      type: "article",
      siteName: "PrinciPL",
      title: absoluteTitle,
      description,
      ...(canonical ? { url: canonical } : {}),
      images: ["/opengraph-image"],
    },
    twitter: {
      card: "summary_large_image",
      title: absoluteTitle,
      description,
      images: ["/twitter-image"],
    },
  };
}
