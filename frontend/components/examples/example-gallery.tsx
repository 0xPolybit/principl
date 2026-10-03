"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { CopyButton } from "@/components/copy-button";
import { PrinciCode } from "@/components/princi-code";
import type { PrinciExample } from "@/content/examples";

const categories = [
  "All examples",
  "Essentials",
  "Control flow",
  "Data & objects",
  "Interop",
] as const;

type CategoryFilter = (typeof categories)[number];

function ExampleCard({ example }: { example: PrinciExample }) {
  return (
    <article className="example-card" id={`example-${example.slug}`}>
      <header className="example-card-heading">
        <div>
          <h2>{example.title}</h2>
          <p className="example-card-file">{example.filename}</p>
        </div>
        <span className={`example-difficulty difficulty-${example.difficulty.toLowerCase()}`}>
          {example.difficulty}
        </span>
      </header>

      <ul aria-label={`${example.title} concepts`} className="example-concepts">
        {example.concepts.map((concept) => (
          <li key={concept}>{concept}</li>
        ))}
      </ul>

      <section aria-label={`${example.title} Princi source`} className="example-code-sample">
        <div className="example-code-heading">
          <span>{example.filename}</span>
          <CopyButton label={`Copy ${example.filename} source`} value={example.source} />
        </div>
        <PrinciCode source={example.source} />
      </section>

      <p className="example-explanation">{example.explanation}</p>

      {example.output !== undefined ? (
        <figure className="example-output">
          <figcaption>
            <span>Static expected output</span>
            <span>stdout</span>
          </figcaption>
          <pre>{example.output}</pre>
        </figure>
      ) : null}

      <nav aria-label={`${example.title} related documentation`} className="example-doc-links">
        {example.docs.map((doc) => (
          <Link href={doc.href} key={doc.href}>
            {doc.label}<span aria-hidden="true"> ↗</span>
          </Link>
        ))}
      </nav>
    </article>
  );
}

export function ExampleGallery({ examples }: { examples: PrinciExample[] }) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<CategoryFilter>("All examples");

  const filteredExamples = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return examples.filter((example) => {
      if (category !== "All examples" && example.category !== category) return false;
      if (!normalizedQuery) return true;
      const searchableText = [
        example.title,
        example.difficulty,
        example.category,
        example.filename,
        example.explanation,
        ...example.concepts,
        example.source,
      ]
        .join(" ")
        .toLowerCase();
      return searchableText.includes(normalizedQuery);
    });
  }, [category, examples, query]);

  function resetFilters() {
    setQuery("");
    setCategory("All examples");
  }

  return (
    <section aria-label="Browse Princi examples" className="examples-gallery">
      <div className="examples-controls">
        <label className="examples-search-label" htmlFor="example-search">
          Search examples
        </label>
        <div className="examples-search-row">
          <svg aria-hidden="true" fill="none" viewBox="0 0 20 20">
            <circle cx="8.7" cy="8.7" r="5.8" />
            <path d="m13 13 4 4" />
          </svg>
          <input
            autoComplete="off"
            id="example-search"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Try “recursion”, “List”, or “class”"
            type="search"
            value={query}
          />
          {query ? (
            <button className="examples-clear-search" onClick={() => setQuery("")} type="button">
              Clear
            </button>
          ) : null}
        </div>

        <div aria-label="Filter examples by category" className="examples-filter-row" role="group">
          {categories.map((entry) => (
            <button
              aria-pressed={category === entry}
              className={category === entry ? "example-filter is-active" : "example-filter"}
              key={entry}
              onClick={() => setCategory(entry)}
              type="button"
            >
              {entry}
            </button>
          ))}
        </div>
        <p aria-live="polite" className="examples-result-count">
          {filteredExamples.length === 1
            ? "1 example"
            : `${filteredExamples.length} examples`}
          {query || category !== "All examples" ? " found" : " in the v0.1 gallery"}
        </p>
      </div>

      {filteredExamples.length > 0 ? (
        <div className="examples-grid">
          {filteredExamples.map((example) => (
            <ExampleCard example={example} key={example.slug} />
          ))}
        </div>
      ) : (
        <div className="examples-empty-state">
          <h2>No matching examples</h2>
          <p>Try a different term or clear the active filters.</p>
          <button className="examples-reset-button" onClick={resetFilters} type="button">
            Show all examples
          </button>
        </div>
      )}
    </section>
  );
}
