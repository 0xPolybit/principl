"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { KeyboardEvent, FormEvent } from "react";
import type { DocsSearchIndex } from "@/lib/docs-search-shared";
import { tokenizeDocsQuery } from "@/lib/docs-search-shared";

export function DocsSearchPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState<DocsSearchIndex | null>(null);
  const [loadingIndex, setLoadingIndex] = useState(false);
  const [indexError, setIndexError] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const resultRefs = useRef<Array<HTMLAnchorElement | null>>([]);
  const indexRequest = useRef<Promise<DocsSearchIndex> | null>(null);

  const loadSearchIndex = useCallback(() => {
    if (index || indexRequest.current) return;

    setIndexError(false);
    setLoadingIndex(true);
    const request = fetch("/docs/search-index", { cache: "force-cache" }).then(async (response) => {
      if (!response.ok) throw new Error("Search index request failed");
      return await response.json() as DocsSearchIndex;
    });
    indexRequest.current = request;
    void request
      .then((loadedIndex) => setIndex(loadedIndex))
      .catch(() => setIndexError(true))
      .finally(() => {
        if (indexRequest.current === request) indexRequest.current = null;
        setLoadingIndex(false);
      });
  }, [index]);

  const openPalette = useCallback(() => {
    setQuery("");
    setOpen(true);
    loadSearchIndex();
  }, [loadSearchIndex]);

  const queryTerms = useMemo(() => tokenizeDocsQuery(query), [query]);
  const results = useMemo(() => {
    if (!index || queryTerms.length === 0) return [];

    const scores = new Map<number, { score: number; matched: number }>();
    for (const term of queryTerms) {
      for (const [targetIndex, weight] of index.postings[term] ?? []) {
        const result = scores.get(targetIndex) ?? { score: 0, matched: 0 };
        result.score += weight;
        result.matched += 1;
        scores.set(targetIndex, result);
      }
    }

    return [...scores.entries()]
      .filter(([, result]) => result.matched === queryTerms.length)
      .map(([targetIndex, result]) => ({ targetIndex, score: result.score }))
      .sort((left, right) => right.score - left.score)
      .slice(0, 12);
  }, [index, queryTerms]);

  useEffect(() => {
    function handleShortcut(event: globalThis.KeyboardEvent) {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openPalette();
      }
    }

    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, [openPalette]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (open && !dialog.open) {
      dialog.showModal();
      window.requestAnimationFrame(() => inputRef.current?.focus());
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  function close() {
    setOpen(false);
  }

  function focusResult(indexToFocus: number) {
    const boundedIndex = Math.max(0, Math.min(indexToFocus, results.length - 1));
    resultRefs.current[boundedIndex]?.focus();
  }

  function handleInputKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "ArrowDown" && results.length > 0) {
      event.preventDefault();
      focusResult(0);
    }
  }

  function handleResultKeyDown(event: KeyboardEvent<HTMLAnchorElement>, indexInResults: number) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      focusResult(indexInResults + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      if (indexInResults === 0) inputRef.current?.focus();
      else focusResult(indexInResults - 1);
    }
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (results.length > 0) resultRefs.current[0]?.click();
  }

  return (
    <>
      <button
        aria-controls="docs-search-dialog"
        aria-expanded={open}
        aria-haspopup="dialog"
        className="docs-search-trigger"
        onClick={openPalette}
        type="button"
      >
        <svg aria-hidden="true" viewBox="0 0 20 20" fill="none">
          <circle cx="8.8" cy="8.8" r="5.8" />
          <path d="m13.2 13.2 4 4" />
        </svg>
        <span>Search documentation</span>
        <kbd aria-hidden="true"><span>⌘</span><span> / Ctrl K</span></kbd>
      </button>

      <dialog
        aria-modal="true"
        aria-labelledby="docs-search-title"
        className="docs-search-dialog"
        id="docs-search-dialog"
        onCancel={(event) => {
          event.preventDefault();
          close();
        }}
        onClose={() => setOpen(false)}
        onClick={(event) => {
          if (event.target === event.currentTarget) close();
        }}
        ref={dialogRef}
      >
        <form className="docs-search-form" onSubmit={handleSubmit} role="search">
          <label className="sr-only" htmlFor="docs-search-input">Search documentation</label>
          <svg aria-hidden="true" viewBox="0 0 20 20" fill="none">
            <circle cx="8.8" cy="8.8" r="5.8" />
            <path d="m13.2 13.2 4 4" />
          </svg>
          <input
            autoComplete="off"
            id="docs-search-input"
            onChange={(event) => setQuery(event.currentTarget.value)}
            onKeyDown={handleInputKeyDown}
            placeholder="Search pages, concepts, and headings..."
            ref={inputRef}
            type="search"
            value={query}
          />
          <button aria-label="Close documentation search" className="docs-search-escape" onClick={close} type="button">
            Esc
          </button>
        </form>
        <h2 className="sr-only" id="docs-search-title">Search PrinciPL documentation</h2>
        <p aria-live="polite" className="docs-search-status">
          {loadingIndex
            ? "Loading the local documentation index…"
            : indexError
              ? "The local index could not be loaded. Try again."
              : queryTerms.length === 0
                ? "Search page titles, descriptions, headings, and guide text."
                : results.length === 0
                  ? "No matching documentation found."
                  : `${results.length} ${results.length === 1 ? "result" : "results"}. Use the arrow keys to browse.`}
        </p>
        {indexError ? (
          <button className="docs-search-retry" onClick={loadSearchIndex} type="button">
            Retry loading the local index
          </button>
        ) : null}
        {results.length > 0 ? (
          <ul aria-label="Documentation search results" className="docs-search-results">
            {results.map((result, resultIndex) => {
              const target = index?.targets[result.targetIndex];
              if (!target) return null;
              return (
                <li key={`${target.href}-${target.kind}`}>
                  <Link
                    onClick={close}
                    onKeyDown={(event) => handleResultKeyDown(event, resultIndex)}
                    ref={(node) => { resultRefs.current[resultIndex] = node; }}
                    href={target.href}
                  >
                    <span className="docs-search-result-title">{target.title}</span>
                    <span className="docs-search-result-context">
                      {target.kind === "heading" ? `${target.pageTitle} · ` : ""}{target.section}
                    </span>
                    <span className="docs-search-result-description">{target.description}</span>
                  </Link>
                </li>
              );
            })}
          </ul>
        ) : null}
        <div className="docs-search-footer">
          <span>Local documentation search</span>
          <span><kbd>↑</kbd> <kbd>↓</kbd> navigate <kbd>Enter</kbd> open</span>
        </div>
      </dialog>
    </>
  );
}
