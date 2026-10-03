import type { Metadata } from "next";
import { DocShell } from "@/components/doc-shell";
import { deferredAreas } from "@/content/princi";
import { pageTitle } from "@/lib/site";

export const metadata: Metadata = {
  title: pageTitle("Roadmap and limits"),
  description:
    "What the Princi v0.1 compiler supports today, what remains intentionally out of scope, and areas documented as possible future work.",
};

const current = [
  "Windows x86-64 native code generation only",
  "Single-command build workflow; no package manager",
  "Built-in io/math import names only",
  "List<T> as the only source generic",
  "Process-lifetime managed heap; no collection during execution",
  "Limited C ABI primitives; no pointers or callbacks",
];

const notSupported = [
  "Linux, macOS, and WebAssembly targets",
  "REPL, formatter, debugger, and IDE integration",
  "Raw pointers, unsafe blocks, ownership, and borrowing",
  "Async/await, traits, interfaces, inheritance, and reflection",
  "Advanced generics, maps, sets, iterators, and Python interoperability",
];

export default function RoadmapPage() {
  return (
    <DocShell
      description="The roadmap starts with an honest boundary. v0.1 is implemented for a focused native target; future directions have no promised schedule."
      note="No dates or delivery promises"
      title="What comes next is still open."
    >
      <section className="article-section">
        <h2>Available in v0.1.</h2>
        <ul>
          {current.map((item) => <li key={item}>{item}</li>)}
        </ul>
        <p>
          See the language guide and compiler architecture for details on the
          capabilities that are actually implemented.
        </p>
      </section>

      <section className="article-section">
        <h2>Explicitly deferred.</h2>
        <ul>
          {notSupported.map((item) => <li key={item}>{item}</li>)}
        </ul>
      </section>

      <section className="article-section">
        <h2>Areas documented as future possibilities.</h2>
        <p>
          The repository mentions these as directions to explore, not
          commitments or scheduled releases:
        </p>
        <ul>
          {deferredAreas.map((item) => <li key={item}>{item}</li>)}
        </ul>
        <div className="callout">
          <strong>Scope follows the compiler.</strong>
          <p>
            The repository&apos;s <code>docs/v0.1-scope.md</code>{" "}defines the current
            boundary. The website does not imply that a deferred feature has
            shipped.
          </p>
        </div>
      </section>
    </DocShell>
  );
}
