import type { Metadata } from "next";
import { DocsDocument } from "@/components/docs/docs-document";
import { docsSections } from "@/content/docs/navigation";
import { pageTitle } from "@/lib/site";

const introduction = docsSections[0].pages[0];

export const metadata: Metadata = {
  title: pageTitle(introduction.title),
  description:
    "A practical guide to Princi v0.1: language syntax, the Windows compiler, runtime, and current limitations.",
};

export default function DocumentationPage() {
  return <DocsDocument doc={introduction} />;
}
