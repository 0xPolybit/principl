import type { Metadata } from "next";
import Link from "next/link";
import { DocShell } from "@/components/doc-shell";
import { docsNavigation, pageTitle } from "@/lib/site";

export const metadata: Metadata = {
  title: pageTitle("Documentation"),
  description:
    "Start with the Princi v0.1 overview, then read the installation, language, architecture, and example guides.",
};

const summaries = [
  ["Installation", "Set up Rust, LLVM/Clang, and MinGW-w64 GCC, then build the compiler."],
  ["Language guide", "Learn the supported v0.1 types, expressions, control flow, and object model."],
  ["Architecture", "Follow the Rust compiler from source loading to a Windows executable."],
  ["Examples", "Explore real syntax for functions, classes, structs, lists, imports, and FFI."],
  ["Roadmap", "See the current boundary and areas deferred beyond v0.1."],
];

export default function DocumentationPage() {
  return (
    <DocShell
      description="A practical guide to what Princi supports today: its source language, Windows build workflow, compiler stages, and deliberate v0.1 limits."
      note="Princi v0.1.0 · Windows x86-64"
      title="Documentation"
    >
      <section className="article-section">
        <h2>Choose a place to begin.</h2>
        <p>
          Princi is an early statically typed language with a native compiler
          written in Rust. It currently builds Windows x86-64 executables from
          either <code>.prnc</code> or <code>.princi</code> source files.
        </p>
        <div className="article-link-list grid grid-cols-1 md:grid-cols-2">
          {summaries.map(([title, description]) => {
            const item = docsNavigation.find((entry) => entry.label === title);
            return item ? (
              <Link className="article-link" href={item.href} key={title}>
                <span>
                  <span className="article-link-title">{title}</span>
                  <span className="block max-w-prose pt-1 text-sm leading-relaxed text-[color:var(--ink-faint)]">
                    {description}
                  </span>
                </span>
                <span aria-hidden="true" className="article-link-arrow">
                  ↗
                </span>
              </Link>
            ) : null;
          })}
        </div>
      </section>
      <section className="article-section">
        <h2>The compiler command</h2>
        <p>
          v0.1 has one user-facing compiler command. With no output override,
          the executable is written beside the source file.
        </p>
        <div className="notice-line">
          <strong>princi build hello.prnc</strong>
          <span>→</span>
          <code>hello.exe</code>
          <span>or choose a path with</span>
          <code>-o app.exe</code>
        </div>
      </section>
      <section className="article-section">
        <h2>Feature claims are versioned.</h2>
        <p>
          Parser support, semantic checks, and native code generation are not
          always the same thing. The guide separates the implemented native
          subset from syntax or future ideas the compiler does not yet lower.
          The repository README and v0.1 scope document remain the detailed
          source of truth.
        </p>
        <Link className="text-link text-link-large" href="/language">
          Read the v0.1 language guide <span aria-hidden="true">↗</span>
        </Link>
      </section>
    </DocShell>
  );
}
