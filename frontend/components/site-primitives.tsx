import Link from "next/link";
import type { PropsWithChildren, ReactNode } from "react";

type ButtonLinkProps = PropsWithChildren<{
  href: string;
  variant?: "primary" | "secondary";
}>;

export function ButtonLink({
  href,
  variant = "primary",
  children,
}: ButtonLinkProps) {
  return (
    <Link className={`button-link button-${variant}`} href={href}>
      {children}
    </Link>
  );
}

export function Badge({
  children,
  tone = "default",
}: PropsWithChildren<{ tone?: "default" | "current" | "roadmap" }>) {
  return <span className={`badge badge-${tone}`}>{children}</span>;
}

export function Card({
  children,
  className = "",
}: PropsWithChildren<{ className?: string }>) {
  return <article className={`ui-card ${className}`.trim()}>{children}</article>;
}

export function Callout({
  children,
  tone = "note",
}: PropsWithChildren<{ tone?: "note" | "caution" }>) {
  return <aside className={`callout callout-${tone}`}>{children}</aside>;
}

export function SectionHeading({
  title,
  description,
}: {
  title: ReactNode;
  description: ReactNode;
}) {
  return (
    <div className="section-header">
      <h2>{title}</h2>
      <p>{description}</p>
    </div>
  );
}

export function FeatureGrid({ children }: PropsWithChildren) {
  return <div className="feature-list feature-grid">{children}</div>;
}

export function InlineCode({ children }: PropsWithChildren) {
  return <code className="inline-code">{children}</code>;
}

export function ArrowUpRight() {
  return (
    <svg aria-hidden="true" className="arrow-icon" fill="none" viewBox="0 0 16 16">
      <path d="M4 12 12 4M5 4h7v7" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" />
    </svg>
  );
}
