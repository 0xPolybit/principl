import { CopyButton } from "@/components/copy-button";

type CodeSampleProps = {
  code: string;
  language?: string;
  title: string;
  filename?: string;
};

export function CodeSample({
  code,
  language = "princi",
  title,
  filename,
}: CodeSampleProps) {
  return (
    <section aria-label={title} className="code-sample">
      <div className="code-sample-heading">
        <div>
          <p className="code-sample-title">{title}</p>
          {filename ? <p className="code-sample-file">{filename}</p> : null}
        </div>
        <CopyButton value={code} />
      </div>
      <pre className="code-sample-body">
        <code className={`language-${language}`}>{code}</code>
      </pre>
    </section>
  );
}
