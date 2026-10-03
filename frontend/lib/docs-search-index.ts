import { docsHref, docsPages } from "@/content/docs/navigation";
import { extractDocsHeadings, extractDocsText, readDocsSource } from "@/lib/docs-content";
import type { DocsSearchIndex, DocsSearchTarget } from "@/lib/docs-search-shared";

function termsIn(value: string): Set<string> {
  return new Set(
    value
      .normalize("NFKD")
      .replace(/\p{M}/gu, "")
      .toLocaleLowerCase("en-US")
      .match(/[\p{L}\p{N}_]+/gu) ?? [],
  );
}

function addField(scores: Map<string, number>, value: string, weight: number) {
  for (const term of termsIn(value)) {
    scores.set(term, Math.max(scores.get(term) ?? 0, weight));
  }
}

let indexPromise: Promise<DocsSearchIndex> | undefined;

/**
 * Build the local search index from the same registry and Markdown files used
 * to render the docs. Next.js evaluates this during static page generation;
 * the index is then serialized to the docs client without a search service.
 */
export function getDocsSearchIndex(): Promise<DocsSearchIndex> {
  indexPromise ??= (async () => {
    const targets: DocsSearchTarget[] = [];
    const postingMap = new Map<string, Array<[targetIndex: number, weight: number]>>();

    async function addTarget(
      target: DocsSearchTarget,
      fields: Array<[value: string, weight: number]>,
    ) {
      const targetIndex = targets.push(target) - 1;
      const scores = new Map<string, number>();
      for (const [value, weight] of fields) addField(scores, value, weight);

      for (const [term, weight] of scores) {
        const entries = postingMap.get(term) ?? [];
        entries.push([targetIndex, weight]);
        postingMap.set(term, entries);
      }
    }

    for (const doc of docsPages) {
      const source = await readDocsSource(doc);
      const baseHref = docsHref(doc);
      const baseFields: Array<[string, number]> = [
        [doc.title, 28],
        [doc.description, 12],
        [doc.section, 5],
        [doc.slug.replaceAll("/", " "), 8],
        [doc.keywords.join(" "), 16],
      ];

      await addTarget(
        {
          href: baseHref,
          title: doc.title,
          pageTitle: doc.title,
          description: doc.description,
          section: doc.section,
          kind: "page",
        },
        [...baseFields, [extractDocsText(source), 1]],
      );

      for (const heading of extractDocsHeadings(source)) {
        await addTarget(
          {
            href: `${baseHref}#${heading.id}`,
            title: heading.title,
            pageTitle: doc.title,
            description: doc.description,
            section: doc.section,
            kind: "heading",
          },
          [
            [heading.title, 30],
            [doc.title, 4],
            [doc.section, 2],
          ],
        );
      }
    }

    return { targets, postings: Object.fromEntries(postingMap) };
  })();

  return indexPromise;
}
