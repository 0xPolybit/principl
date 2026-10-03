import Link from "next/link";
import { ArrowUpRight } from "@/components/site-primitives";
import { DocsSidebar } from "@/components/docs/docs-sidebar";
import { DocsTableOfContents } from "@/components/docs/docs-toc";
import {
  docsHref,
  docsNeighbors,
  docsSections,
  findDocsSection,
  type DocsPage,
} from "@/content/docs/navigation";
import type { DocsHeading } from "@/lib/docs-content";
import { repositoryUrl } from "@/lib/site";

function Breadcrumbs({ doc }: { doc: DocsPage }) {
  const section = findDocsSection(doc);
  return (
    <nav aria-label="Breadcrumb" className="docs-breadcrumbs">
      <ol>
        <li><Link href="/docs">Documentation</Link></li>
        {section ? <li><span aria-current="page">{section.title}</span></li> : null}
        <li><span aria-current="page">{doc.title}</span></li>
      </ol>
    </nav>
  );
}

function PageNavigation({ doc }: { doc: DocsPage }) {
  const { previous, next } = docsNeighbors(doc);
  return (
    <nav aria-label="Previous and next documentation pages" className="docs-page-navigation">
      {previous ? (
        <Link className="docs-page-link docs-page-previous" href={docsHref(previous)} rel="prev">
          <span>Previous</span>
          <strong>{previous.title}</strong>
        </Link>
      ) : <span />}
      {next ? (
        <Link className="docs-page-link docs-page-next" href={docsHref(next)} rel="next">
          <span>Next</span>
          <strong>{next.title}</strong>
        </Link>
      ) : <span />}
    </nav>
  );
}

export function DocsLayout({
  doc,
  headings,
  children,
}: {
  doc: DocsPage;
  headings: DocsHeading[];
  children: React.ReactNode;
}) {
  const editUrl = repositoryUrl + "/edit/main/frontend/content/docs/" + doc.file;

  return (
    <div className="docs-layout page-width">
      <DocsSidebar activeSlug={doc.slug} sections={docsSections} />
      <main className="docs-main" id="main-content">
        <Breadcrumbs doc={doc} />
        {headings.length > 0 ? (
          <div className="docs-mobile-toc">
            <details>
              <summary>On this page</summary>
              <DocsTableOfContents headings={headings} />
            </details>
          </div>
        ) : null}
        <article className="docs-article">
          <header className="docs-page-header">
            <div className="docs-title-row">
              <h1>{doc.title}</h1>
              <span className="docs-version">v0.1</span>
            </div>
            <p>{doc.description}</p>
          </header>
          {children}
          <PageNavigation doc={doc} />
          <div className="docs-edit-link">
            <a href={editUrl} rel="noreferrer" target="_blank">
              Edit this page on GitHub <ArrowUpRight />
            </a>
          </div>
        </article>
      </main>
      <DocsTableOfContents headings={headings} />
    </div>
  );
}
