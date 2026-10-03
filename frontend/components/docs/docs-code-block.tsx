import { Children, isValidElement, type ReactNode } from "react";
import { CopyButton } from "@/components/copy-button";

function codeLanguage(children: ReactNode) {
  const code = Children.toArray(children).find((child) => isValidElement(child));
  if (!isValidElement<{ className?: string }>(code)) return "Code";
  const match = code.props.className?.match(/language-([\w-]+)/);
  return match ? match[1] : "Code";
}

export function DocsCodeBlock({ children }: { children?: ReactNode }) {
  return (
    <figure className="docs-code-block">
      <figcaption>
        <span>{codeLanguage(children)}</span>
        <CopyButton readCodeBlock value="" />
      </figcaption>
      <pre>{children}</pre>
    </figure>
  );
}
