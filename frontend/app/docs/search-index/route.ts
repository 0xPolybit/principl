import { getDocsSearchIndex } from "@/lib/docs-search-index";

export const dynamic = "force-static";

export async function GET() {
  const searchIndex = await getDocsSearchIndex();
  return Response.json(searchIndex, {
    headers: {
      "Cache-Control": "public, max-age=3600, stale-while-revalidate=86400",
      "X-Robots-Tag": "noindex",
    },
  });
}
