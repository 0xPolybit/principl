import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { DocsDocument } from "@/components/docs/docs-document";
import { docsRouteParams, findDocsPage } from "@/content/docs/navigation";
import { pageTitle } from "@/lib/site";

type DocsRouteProps = {
  params: Promise<{ slug: string[] }>;
};

export const dynamicParams = false;

export function generateStaticParams() {
  return docsRouteParams();
}

async function pageForParams(params: DocsRouteProps["params"]) {
  const { slug } = await params;
  return findDocsPage(slug.join("/"));
}

export async function generateMetadata({
  params,
}: DocsRouteProps): Promise<Metadata> {
  const doc = await pageForParams(params);
  if (!doc) notFound();
  return {
    title: pageTitle(doc.title),
    description: doc.description,
  };
}

export default async function DocumentationEntry({ params }: DocsRouteProps) {
  const doc = await pageForParams(params);
  if (!doc) notFound();
  return <DocsDocument doc={doc} />;
}
