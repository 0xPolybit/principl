import { readFile } from "node:fs/promises";
import path from "node:path";
import GithubSlugger from "github-slugger";
import { remark } from "remark";
import remarkGfm from "remark-gfm";
import type { DocsPage } from "@/content/docs/navigation";

type MarkdownNode = {
  type: string;
  depth?: number;
  value?: string;
  children?: MarkdownNode[];
};

export type DocsHeading = {
  id: string;
  title: string;
  depth: number;
};

export async function readDocsSource(doc: DocsPage) {
  const basePath = path.join(process.cwd(), "content", "docs");
  return readFile(path.join(basePath, doc.file), "utf8");
}

function nodeText(node: MarkdownNode): string {
  if (node.type === "image") return "";
  if (node.value) return node.value;
  return (node.children ?? []).map(nodeText).join("");
}

export function extractDocsHeadings(source: string): DocsHeading[] {
  const tree = remark().use(remarkGfm).parse(source) as unknown as MarkdownNode;
  const slugger = new GithubSlugger();
  const headings: DocsHeading[] = [];

  function visit(node: MarkdownNode) {
    if (node.type === "heading" && node.depth && node.depth > 1 && node.depth < 4) {
      const title = nodeText(node).trim();
      if (title) headings.push({ id: slugger.slug(title), title, depth: node.depth });
      return;
    }
    for (const child of node.children ?? []) visit(child);
  }

  visit(tree);
  return headings;
}

export function extractDocsText(source: string): string {
  const tree = remark().use(remarkGfm).parse(source) as unknown as MarkdownNode;

  function collect(node: MarkdownNode): string {
    if (node.type === "image") return "";
    if (node.value) return node.value;
    return (node.children ?? []).map(collect).join(" ");
  }

  return collect(tree).replace(/<!--[\s\S]*?-->/g, " ").replace(/\s+/g, " ").trim();
}
