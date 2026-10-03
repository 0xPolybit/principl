"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import type { DocsHeading } from "@/lib/docs-content";

function HeadingLinks({ headings, activeId }: { headings: DocsHeading[]; activeId: string }) {
  return (
    <ol>
      {headings.map((heading) => (
        <li className={heading.depth === 3 ? "docs-toc-subheading" : undefined} key={heading.id}>
          <Link aria-current={activeId === heading.id ? "location" : undefined} href={"#" + heading.id}>
            {heading.title}
          </Link>
        </li>
      ))}
    </ol>
  );
}

export function DocsTableOfContents({ headings }: { headings: DocsHeading[] }) {
  const [activeId, setActiveId] = useState(headings[0]?.id ?? "");

  useEffect(() => {
    if (headings.length === 0) return;
    let frame = 0;

    function updateActiveHeading() {
      window.cancelAnimationFrame(frame);
      frame = window.requestAnimationFrame(() => {
        let current = headings[0];
        for (const heading of headings) {
          const element = document.getElementById(heading.id);
          if (element && element.getBoundingClientRect().top <= window.innerHeight * 0.3) current = heading;
          else break;
        }
        setActiveId(current.id);
      });
    }

    const observer = new IntersectionObserver(updateActiveHeading, {
      rootMargin: "-12% 0px -72% 0px",
      threshold: 0,
    });
    for (const heading of headings) {
      const element = document.getElementById(heading.id);
      if (element) observer.observe(element);
    }
    window.addEventListener("scroll", updateActiveHeading, { passive: true });
    window.addEventListener("resize", updateActiveHeading);
    updateActiveHeading();

    return () => {
      observer.disconnect();
      window.removeEventListener("scroll", updateActiveHeading);
      window.removeEventListener("resize", updateActiveHeading);
      window.cancelAnimationFrame(frame);
    };
  }, [headings]);

  if (headings.length === 0) return null;

  return (
    <aside aria-label="Page outline" className="docs-toc">
      <nav aria-label="On this page">
        <h2>On this page</h2>
        <HeadingLinks activeId={activeId} headings={headings} />
      </nav>
    </aside>
  );
}
