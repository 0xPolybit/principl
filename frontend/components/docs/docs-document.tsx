import { Fragment, type ReactNode } from "react";
import { DocsLayout } from "@/components/docs/docs-layout";
import { MarkdownContent } from "@/components/docs/markdown-content";
import { ReleaseDownloads } from "@/components/release-downloads";
import {
  CompilerPipelineDiagram,
  MemoryRoadmapDiagram,
} from "@/components/docs/compiler-diagrams";
import type { DocsPage } from "@/content/docs/navigation";
import { extractDocsHeadings, readDocsSource } from "@/lib/docs-content";

const embeddedBlocks: Record<string, ReactNode> = {
  "<!-- princi-diagram:compiler-pipeline -->": <CompilerPipelineDiagram />,
  "<!-- princi-diagram:memory-roadmap -->": <MemoryRoadmapDiagram />,
  "<!-- princi-release-downloads -->": <ReleaseDownloads />,
};

export async function DocsDocument({ doc }: { doc: DocsPage }) {
  const source = await readDocsSource(doc);
  const headings = extractDocsHeadings(source);
  const content = source.split(/(<!-- princi-(?:diagram:[a-z-]+|release-downloads) -->)/g);

  return (
    <DocsLayout doc={doc} headings={headings}>
      {content.map((part, index) => {
        const embeddedBlock = embeddedBlocks[part];
        if (embeddedBlock) return <Fragment key={index}>{embeddedBlock}</Fragment>;
        if (part.startsWith("<!-- princi-") || !part.trim()) return null;
        return <MarkdownContent key={index} source={part} />;
      })}
    </DocsLayout>
  );
}
