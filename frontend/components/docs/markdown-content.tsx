import type { Components } from "react-markdown";
import ReactMarkdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import rehypeSlug from "rehype-slug";
import remarkGfm from "remark-gfm";
import { DocsCodeBlock } from "@/components/docs/docs-code-block";

const components: Components = {
  a: ({ href, children, ...props }) => (
    <a href={href} rel={href?.startsWith("http") ? "noreferrer" : undefined} {...props}>
      {children}
    </a>
  ),
  blockquote: ({ children }) => <aside className="docs-callout">{children}</aside>,
  h2: ({ children, id }) => (
    <h2 className="docs-heading" id={id}>
      <a aria-label="Link to this section" className="docs-heading-anchor" href={"#" + id}>#</a>
      {children}
    </h2>
  ),
  h3: ({ children, id }) => (
    <h3 className="docs-heading" id={id}>
      <a aria-label="Link to this section" className="docs-heading-anchor" href={"#" + id}>#</a>
      {children}
    </h3>
  ),
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
