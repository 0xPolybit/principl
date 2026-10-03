import type { Components } from "react-markdown";
import { Children, isValidElement, type ReactNode } from "react";
import ReactMarkdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import rehypeSlug from "rehype-slug";
import remarkGfm from "remark-gfm";
import { CopyHeadingLink } from "@/components/docs/copy-heading-link";
import { DocsCodeBlock } from "@/components/docs/docs-code-block";

function textFromChildren(children: ReactNode): string {
  return Children.toArray(children).map((child) => {
    if (typeof child === "string" || typeof child === "number") return String(child);
    if (isValidElement<{ children?: ReactNode }>(child)) return textFromChildren(child.props.children);
    return "";
  }).join("").trim();
}

function DocsHeading({
  as: Heading,
  children,
  id,
}: {
  as: "h2" | "h3";
  children?: ReactNode;
  id?: string;
}) {
  const title = textFromChildren(children);
  return (
    <Heading className="docs-heading" id={id}>
      {id ? <a aria-label={`Link to ${title || "this section"}`} className="docs-heading-anchor" href={`#${id}`}>#</a> : null}
      {children}
      {id && title ? <CopyHeadingLink heading={title} id={id} /> : null}
    </Heading>
  );
}

const components: Components = {
  a: ({ href, children, ...props }) => (
    <a href={href} rel={href?.startsWith("http") ? "noreferrer" : undefined} {...props}>
      {children}
    </a>
  ),
  blockquote: ({ children }) => <aside className="docs-callout">{children}</aside>,
  h2: ({ children, id }) => <DocsHeading as="h2" id={id}>{children}</DocsHeading>,
  h3: ({ children, id }) => <DocsHeading as="h3" id={id}>{children}</DocsHeading>,
  pre: ({ children }) => <DocsCodeBlock>{children}</DocsCodeBlock>,
  table: ({ children }) => (
    <div className="docs-table-scroll">
      <table>{children}</table>
    </div>
  ),
};

export function MarkdownContent({ source }: { source: string }) {
  return (
    <div className="docs-prose">
      <ReactMarkdown
        components={components}
        rehypePlugins={[
          rehypeSlug,
          [rehypeHighlight, { aliases: { rust: ["princi"] } }],
          rehypeHighlightCodeLines,
        ]}
        remarkPlugins={[remarkGfm]}
      >
        {source}
      </ReactMarkdown>
    </div>
  );
}
