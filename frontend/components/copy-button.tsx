"use client";

import { useRef, useState } from "react";

export function CopyButton({
  value,
  label = "Copy code",
  readCodeBlock = false,
}: {
  value: string;
  label?: string;
  readCodeBlock?: boolean;
}) {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");
  const buttonRef = useRef<HTMLButtonElement>(null);

  async function copyText() {
    try {
      const code = readCodeBlock
        ? buttonRef.current?.closest(".docs-code-block")?.querySelector("pre code")?.textContent
        : undefined;
      await navigator.clipboard.writeText(code ?? value);
      setStatus("copied");
      window.setTimeout(() => setStatus("idle"), 1800);
    } catch {
      setStatus("failed");
      window.setTimeout(() => setStatus("idle"), 2400);
    }
  }

  const buttonText =
    status === "copied"
      ? "Copied"
      : status === "failed"
        ? "Copy unavailable"
        : "Copy";
  const accessibleLabel =
    status === "copied"
      ? `${label} copied`
      : status === "failed"
        ? `${label} unavailable`
        : label;

  return (
    <button aria-label={accessibleLabel} className="copy-button" onClick={copyText} ref={buttonRef} type="button">
      <span aria-live="polite">{buttonText}</span>
      <svg aria-hidden="true" viewBox="0 0 16 16" fill="none">
        <rect x="5.3" y="2.3" width="8.2" height="10.3" rx="1.5" />
        <path d="M10.7 12.7v.8a1.2 1.2 0 0 1-1.2 1.2H3.1a1.2 1.2 0 0 1-1.2-1.2V5.8a1.2 1.2 0 0 1 1.2-1.2h.8" />
      </svg>
    </button>
  );
}
