"use client";

import { useState } from "react";

export function CopyButton({ value }: { value: string }) {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");

  async function copyText() {
    try {
      await navigator.clipboard.writeText(value);
      setStatus("copied");
      window.setTimeout(() => setStatus("idle"), 1800);
    } catch {
      setStatus("failed");
      window.setTimeout(() => setStatus("idle"), 2400);
    }
  }

  const label =
    status === "copied"
      ? "Copied"
      : status === "failed"
        ? "Copy unavailable"
        : "Copy";

  return (
    <button className="copy-button" onClick={copyText} type="button">
      <span aria-live="polite">{label}</span>
      <svg aria-hidden="true" viewBox="0 0 16 16" fill="none">
        <rect x="5.3" y="2.3" width="8.2" height="10.3" rx="1.5" />
        <path d="M10.7 12.7v.8a1.2 1.2 0 0 1-1.2 1.2H3.1a1.2 1.2 0 0 1-1.2-1.2V5.8a1.2 1.2 0 0 1 1.2-1.2h.8" />
      </svg>
    </button>
  );
}
