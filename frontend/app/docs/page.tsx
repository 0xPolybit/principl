import { DocsDocument } from "@/components/docs/docs-document";
import { docsSections } from "@/content/docs/navigation";
import { docsMetadata } from "@/lib/docs-metadata";

const introduction = docsSections[0].pages[0];

export const metadata = docsMetadata(introduction.title, introduction.description, "/docs");

export default function DocumentationPage() {
  return <DocsDocument doc={introduction} />;
}
