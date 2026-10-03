import { DocsLayout } from "@/components/docs/docs-layout";
import { MarkdownContent } from "@/components/docs/markdown-content";
import type { DocsPage } from "@/content/docs/navigation";
import { extractDocsHeadings, readDocsSource } from "@/lib/docs-content";

export async function DocsDocument({ doc }: { doc: DocsPage }) {
  const source = await readDocsSource(doc);
  const headings = extractDocsHeadings(source);

  return (
    <DocsLayout doc={doc} headings={headings}>
      <MarkdownContent source={source} />
    </DocsLayout>
  );
}
