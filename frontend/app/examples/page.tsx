import type { Metadata } from "next";
import { CodeSample } from "@/components/code-sample";
import { DocShell } from "@/components/doc-shell";
import {
  classSource,
  factorialSource,
  ffiSource,
  helloSource,
  listSource,
  moduleSource,
  structSource,
} from "@/content/princi";
import { pageTitle } from "@/lib/site";

export const metadata: Metadata = {
  title: pageTitle("Examples"),
  description:
    "Copyable Princi v0.1 examples for hello world, recursion, classes, structs, lists, imports, and the limited C FFI.",
};

const sections = [
  ["hello", "First program"],
  ["functions", "Functions and recursion"],
  ["classes", "Classes"],
  ["structs", "Value-oriented structs"],
  ["lists", "List<T>"],
  ["imports", "Built-in imports"],
  ["ffi", "Limited C FFI"],
];

export default function ExamplesPage() {
  return (
    <DocShell
      description="These short programs use syntax covered by the v0.1 compiler. Copy a sample, save it as .prnc or .princi, and build it on Windows."
      note="Copyable source snippets"
      title="Princi examples"
    >
      <nav aria-label="Example sections" className="example-index">
        {sections.map(([id, title]) => (
          <a href={`#${id}`} key={id}>
            {title}
          </a>
        ))}
      </nav>

      <section className="example-section" id="hello">
        <h2>First program</h2>
        <p>Printing a string works in the smallest complete program.</p>
        <CodeSample code={helloSource} title="Hello from Princi" filename="hello.prnc" />
      </section>

      <section className="example-section" id="functions">
        <h2>Functions and recursion</h2>
        <p>Parameters and return values are statically checked.</p>
        <CodeSample code={factorialSource} title="Factorial" filename="factorial.prnc" />
      </section>

      <section className="example-section" id="classes">
        <h2>Classes</h2>
        <p>Class values are managed references; method dispatch is static.</p>
        <CodeSample code={classSource} title="User class" filename="user.prnc" />
      </section>

      <section className="example-section" id="structs">
        <h2>Value-oriented structs</h2>
        <p>Struct assignments, parameters, and return values copy the fields.</p>
        <CodeSample code={structSource} title="Point struct" filename="point.prnc" />
      </section>

      <section className="example-section" id="lists">
        <h2>List&lt;T&gt;</h2>
        <p>
          A list tracks one checked element type. Indexing is bounds checked at
          runtime.
        </p>
        <CodeSample code={listSource} title="Mutable integer list" filename="numbers.prnc" />
      </section>

      <section className="example-section" id="imports">
        <h2>Built-in imports</h2>
        <p>
          The resolver accepts <code>io</code> and <code>math</code>; <code>math</code> is currently registered
          but has no exports. Imports do not create namespaces.
        </p>
        <CodeSample code={moduleSource} title="Standard module names" filename="modules.prnc" />
      </section>

      <section className="example-section" id="ffi">
        <h2>Limited C FFI</h2>
        <p>
          This example declares a C runtime symbol with an FFI-safe integer
          signature. Managed values and pointer types are not permitted.
        </p>
        <CodeSample code={ffiSource} title="C runtime abs" filename="abs.prnc" />
      </section>
    </DocShell>
  );
}
