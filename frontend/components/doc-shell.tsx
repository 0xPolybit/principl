import { ArticleHeader } from "@/components/article-header";

type DocShellProps = {
  title: string;
  description: string;
  note?: string;
  children: React.ReactNode;
};

export function DocShell({ title, description, note, children }: DocShellProps) {
  return (
    <main className="article-page" id="main-content">
      <div className="reading-width">
        <ArticleHeader title={title} description={description} note={note} />
      </div>
      <div className="page-width article-content">{children}</div>
    </main>
  );
}
