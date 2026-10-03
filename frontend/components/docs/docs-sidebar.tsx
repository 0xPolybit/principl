"use client";

import Link from "next/link";
import { useRef } from "react";
import { docsHref, type DocsPage, type DocsSection } from "@/content/docs/navigation";

function NavigationGroups({
  sections,
  activeSlug,
  onNavigate,
}: {
  sections: DocsSection[];
  activeSlug: string;
  onNavigate?: () => void;
}) {
  return (
    <>
      {sections.map((section) => (
        <details
          className="docs-nav-group"
          key={section.title}
          open={section.pages.some((doc) => doc.slug === activeSlug)}
        >
          <summary>{section.title}</summary>
          <ul>
            {section.pages.map((doc: DocsPage) => (
              <li key={doc.slug || "introduction"}>
                <Link
                  aria-current={doc.slug === activeSlug ? "page" : undefined}
                  className="docs-nav-link"
                  href={docsHref(doc)}
                  onClick={onNavigate}
                >
                  {doc.title}
                </Link>
              </li>
            ))}
          </ul>
        </details>
      ))}
    </>
  );
}

export function DocsSidebar({
  sections,
  activeSlug,
}: {
  sections: DocsSection[];
  activeSlug: string;
}) {
  const drawerRef = useRef<HTMLDetailsElement>(null);
  const closeDrawer = () => {
    if (drawerRef.current) drawerRef.current.open = false;
  };

  return (
    <aside className="docs-sidebar">
      <details className="docs-mobile-drawer" ref={drawerRef}>
        <summary>Browse documentation</summary>
        <nav aria-label="Documentation navigation" className="docs-nav">
          <NavigationGroups
            activeSlug={activeSlug}
            onNavigate={closeDrawer}
            sections={sections}
          />
        </nav>
      </details>
      <nav aria-label="Documentation navigation" className="docs-nav docs-desktop-nav">
        <NavigationGroups activeSlug={activeSlug} sections={sections} />
      </nav>
    </aside>
  );
}
