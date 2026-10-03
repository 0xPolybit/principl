import Link from "next/link";
import type { DocsHeading } from "@/lib/docs-content";

function HeadingLinks({ headings }: { headings: DocsHeading[] }) {
  return (
    <ol>
      {headings.map((heading) => (
        <li className={heading.depth === 3 ? "docs-toc-subheading" : undefined} key={heading.id}>
          <Link href={"#" + heading.id}>{heading.title}</Link>
        </li>
      ))}
    </ol>
  );
}

export function DocsTableOfContents({ headings }: { headings: DocsHeading[] }) {
  if (headings.length === 0) return null;

  return (
    <aside aria-label="Page outline" className="docs-toc">
      <nav aria-label="On this page">
        <h2>On this page</h2>
        <HeadingLinks headings={headings} />
      </nav>
    </aside>
  );
}
