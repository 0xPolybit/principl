"use client";

import { useState } from "react";

export function CopyHeadingLink({ heading, id }: { heading: string; id: string }) {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");

  async function copyLink() {
    try {
      const link = new URL(window.location.href);
      link.hash = id;
      await navigator.clipboard.writeText(link.toString());
      setStatus("copied");
      window.setTimeout(() => setStatus("idle"), 1800);
    } catch {
      setStatus("failed");
      window.setTimeout(() => setStatus("idle"), 2400);
    }
  }

  const label = status === "copied"
    ? "Heading link copied"
    : status === "failed"
      ? "Heading link unavailable"
      : `Copy link to ${heading}`;

  return (
    <button
      aria-label={label}
      className="docs-heading-copy"
      onClick={copyLink}
      title={label}
      type="button"
    >
      <span aria-live="polite" className="sr-only">{status === "copied" ? "Link copied" : status === "failed" ? "Could not copy link" : ""}</span>
      <svg aria-hidden="true" viewBox="0 0 16 16" fill="none">
        <path d="M6.25 9.75 9.75 6.25" />
        <path d="M5.15 11.3H4.4a2.8 2.8 0 0 1 0-5.6h2.1M10.85 4.7h.75a2.8 2.8 0 0 1 0 5.6H9.5" />
      </svg>
    </button>
  );
}
