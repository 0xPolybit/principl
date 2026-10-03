import type { Metadata } from "next";
import { CodeSample } from "@/components/code-sample";
import { DocShell } from "@/components/doc-shell";
import { classSource, listSource, structSource } from "@/content/princi";
import { pageTitle } from "@/lib/site";

export const metadata: Metadata = {
  title: pageTitle("Language guide"),
  description:
    "The implemented Princi v0.1 language: primitive types, local bindings, functions, control flow, classes, structs, lists, imports, and limited C FFI.",
};

const operators = [
  ["Arithmetic", "+  -  *  /  %", "Matching Int or Float operands; % is Int-only. String + concatenates."],
  ["Comparison", "==  !=  <  <=  >  >=", "Matching numeric types; equality also supports matching primitive values."],
  ["Boolean", "&&  ||  !", "Boolean operands and conditions must have type Bool."],
];

export default function LanguagePage() {
  return (
    <DocShell
      description="The v0.1 language is statically typed, local in its inference, and deliberately specific about what compiles to native code."
      note="Implemented native subset"
      title="The Princi language"
    >
      <section className="article-section">
        <h2>Types and local bindings.</h2>
        <p>
          Primitive types are <code>Int</code> (signed 64-bit), <code>Float</code> (64-bit), <code>Bool</code>,{" "}
          <code>String</code>, and <code>Void</code>. Classes and structs introduce nominal types.
          <code>List&lt;T&gt;</code> is the only generic source type in v0.1.
        </p>
        <p>
          <code>let</code> is immutable and needs an initializer. <code>var</code> is mutable. A
          local can infer its type from a simple initializer, and a variable
          declared without a value must be assigned before it is read. There
          are no implicit numeric conversions.
        </p>
        <div className="notice-line">
          <strong>var count = 10</strong>
          <span>→</span>
          <code>Int</code>
          <strong>let label = &quot;hello&quot;</strong>
          <span>→</span>
          <code>String</code>
        </div>
      </section>

      <section className="article-section">
        <h2>Functions and control flow.</h2>
        <p>
          Functions have typed parameters and optional return annotations.
          Omitted return types mean <code>Void</code>. Non-<code>Void</code> functions must return
          on every control-flow path. v0.1 supports recursion, <code>if</code>/<code>else</code>,
          <code>while</code>, and exclusive-end integer range loops.
        </p>
        <CodeSample
          code={`fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }

    return n * factorial(n - 1)
}

fn main() {
    println(factorial(5))
}`}
          title="Recursion and branching"
          filename="factorial.prnc"
        />
        <p>
          Semicolons may separate statements. Line breaks also separate
          statements when the next token cannot continue the current
          expression.
        </p>
      </section>

      <section className="article-section">
        <h2>Expressions and operators.</h2>
        <p>
          Expressions include primitive literals, calls, member access,
          indexing, list literals, unary operators, and binary expressions.
          Operator precedence follows the parser&apos;s published table.
        </p>
        <table className="boundary-table">
          <thead>
            <tr>
              <th scope="col">Family</th>
              <th scope="col">Operators</th>
              <th scope="col">Rule</th>
            </tr>
          </thead>
          <tbody>
            {operators.map(([family, symbols, rule]) => (
              <tr key={family}>
                <th scope="row">{family}</th>
                <td><code>{symbols}</code></td>
                <td>{rule}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="article-section">
        <h2>Classes and structs.</h2>
        <p>
          Classes are managed references with fields, one optional <code>init</code>, and
          statically dispatched methods. Visibility modifiers and inheritance
          are not implemented. Structs are field-only values: assignment,
          parameters, and returns copy the aggregate.
        </p>
        <CodeSample code={classSource} title="A class with an initializer" filename="user.prnc" />
        <CodeSample code={structSource} title="A struct passed by value" filename="point.prnc" />
      </section>

      <section className="article-section">
        <h2>List&lt;T&gt;.</h2>
        <p>
          List literals must have one element type; an empty list needs an
          explicit <code>List&lt;T&gt;</code> type. Lists are reference values. <code>.length</code>,{" "}
          indexing, indexed assignment, and <code>.add(value)</code> are supported. The
          runtime checks index bounds.
        </p>
        <CodeSample code={listSource} title="Typed list operations" filename="numbers.prnc" />
      </section>

      <section className="article-section">
        <h2>Imports and C declarations.</h2>
        <p>
          Only built-in <code>io</code> and <code>math</code> imports resolve. <code>io</code> is optional for
          <code>print</code> and <code>println</code>;{" "}<code>math</code> currently exports no functions. There
          is no user-defined module or package lookup.
        </p>
        <p>
          <code>extern &quot;C&quot;</code> accepts only <code>Int32</code>, <code>Int64</code>, and <code>Float64</code>
          parameters, plus <code>Void</code> returns. Managed values, pointers, callbacks,
          variadic functions, and custom link flags are outside v0.1.
        </p>
      </section>

      <section className="article-section">
        <h2>Source files stay interchangeable.</h2>
        <p>
          Save the same source as <code>.prnc</code> or <code>.princi</code>. Both extensions select
          the same lexer, parser, type rules, and generated program. By default{" "}
          <code>princi build hello.prnc</code> writes <code>hello.exe</code> beside the source.
        </p>
      </section>

      <section className="article-section">
        <h2>Not part of v0.1.</h2>
        <p>
          The compiler does not support Linux, macOS, WebAssembly, a package
          manager, REPL, formatter, debugger, IDE integration, pointers, unsafe
          blocks, ownership, borrowing, async/await, traits, inheritance,
          advanced generics, reflection, or Python interoperability.
        </p>
      </section>
    </DocShell>
  );
}
