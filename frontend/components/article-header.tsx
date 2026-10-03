export function ArticleHeader({
  title,
  description,
  note,
}: {
  title: string;
  description: string;
  note?: string;
}) {
  return (
    <header className="article-header">
      <h1>{title}</h1>
      <p>{description}</p>
      {note ? <span className="article-note">{note}</span> : null}
    </header>
  );
}
