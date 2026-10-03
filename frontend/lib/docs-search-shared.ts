export type DocsSearchTarget = {
  href: string;
  title: string;
  pageTitle: string;
  description: string;
  section: string;
  kind: "page" | "heading";
};

export type DocsSearchIndex = {
  targets: DocsSearchTarget[];
  postings: Record<string, Array<[targetIndex: number, weight: number]>>;
};

export function tokenizeDocsQuery(query: string): string[] {
  return [
    ...new Set(
      query
        .normalize("NFKD")
        .replace(/\p{M}/gu, "")
        .toLocaleLowerCase("en-US")
        .match(/[\p{L}\p{N}_]+/gu) ?? [],
    ),
  ].filter((term) => term.length > 1);
}
