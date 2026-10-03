import { Fragment, type ReactNode } from "react";
import { DocsLayout } from "@/components/docs/docs-layout";
import { MarkdownContent } from "@/components/docs/markdown-content";
import {
  CompilerPipelineDiagram,
  MemoryRoadmapDiagram,
} from "@/components/docs/compiler-diagrams";
import type { DocsPage } from "@/content/docs/navigation";
import { extractDocsHeadings, readDocsSource } from "@/lib/docs-content";

const diagrams: Record<string, ReactNode> = {
  "<!-- princi-diagram:compiler-pipeline -->": <CompilerPipelineDiagram />,
  "<!-- princi-diagram:memory-roadmap -->": <MemoryRoadmapDiagram />,
};

export async function DocsDocument({ doc }: { doc: DocsPage }) {
  const source = await readDocsSource(doc);
  const headings = extractDocsHeadings(source);
  const content = source.split(/(<!-- princi-diagram:[a-z-]+ -->)/g);

  return (
    <DocsLayout doc={doc} headings={headings}>
      {content.map((part, index) => {
        const diagram = diagrams[part];
        if (diagram) return <Fragment key={index}>{diagram}</Fragment>;
        if (part.startsWith("<!-- princi-diagram:") || !part.trim()) return null;
        return <MarkdownContent key={index} source={part} />;
      })}
    </DocsLayout>
  );
}
