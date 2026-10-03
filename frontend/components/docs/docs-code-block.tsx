import { Children, isValidElement, type ReactNode } from "react";
import { CopyButton } from "@/components/copy-button";

function textFromNode(node: ReactNode): string {
  if (typeof node === "string" || typeof node === "number") return String(node);
  return Children.toArray(node).map((child) => {
    if (isValidElement<{ children?: ReactNode }>(child)) {
      return textFromNode(child.props.children);
    }
    return "";
  }).join("");
}

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
        <CopyButton value={textFromNode(children)} />
      </figcaption>
      <pre>{children}</pre>
    </figure>
  );
}
