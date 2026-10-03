import Link from "next/link";
import {
  ArrowUpRight,
  Badge,
  ButtonLink,
  Callout,
  SectionHeading,
} from "@/components/site-primitives";
import { CodeSample } from "@/components/code-sample";
import {
  classSource,
  controlFlowSource,
  factorialSource,
  functionSource,
  listSource,
  nativePipeline,
  structCopySource,
  variablesSource,
} from "@/content/princi";
import { repositoryUrl } from "@/lib/site";

const languageSamples = [
  {
    title: "Bindings",
    filename: "values.prnc",
    code: variablesSource,
  },
  {
    title: "Functions and calls",
    filename: "functions.prnc",
    code: functionSource,
  },
  {
    title: "Conditions and loops",
    filename: "control.prnc",
    code: controlFlowSource,
  },
  {
    title: "Classes",
    filename: "user.prnc",
    code: classSource,
  },
  {
    title: "Struct values",
    filename: "point.prnc",
    code: structCopySource,
  },
  {
    title: "List<T>",
    filename: "numbers.prnc",
    code: listSource,
  },
];

const shippedCapabilities = [
  {
    title: "Typed program core",
    description:
      "Int, Float, Bool, String, local inference, functions, recursion, assignments, and checked expressions.",
  },
  {
    title: "Control flow and data",
    description:
      "if/else, while, integer ranges, reference-oriented classes, value-oriented structs, and List<T>.",
  },
  {
    title: "Native build and runtime",
    description:
      "print/println, a limited C ABI, verified LLVM IR, and Windows x86-64 executable linking.",
  },
];

const roadmapAreas = [
  {
    title: "Richer memory model",
    description:
      "Tracing collection is a future possibility. v0.1 keeps managed allocations until main returns; it does not collect during execution.",
  },
  {
    title: "Ownership and raw memory",
    description:
      "Ownership, borrowing, raw pointers, and unsafe blocks are not part of the current language.",
  },
  {
    title: "More targets and language tools",
    description:
      "Linux, macOS, WebAssembly, package management, richer generics, async, and traits remain deferred.",
  },
];

export default function Home() {
  return (
    <main id="main-content">
      <section className="page-width landing-hero">
        <div className="landing-hero-copy">
          <div className="landing-hero-status">
            <Badge tone="current">Princi v0.1</Badge>
            <span>Windows x86-64</span>
          </div>
          <h1>
            Structure when needed.
            <br />
            Simplicity by default.
          </h1>
          <p className="landing-hero-description">
            PrinciPL is a statically typed, compiled programming language. The
            v0.1 compiler turns concise, structured source into native Windows
            executables.
          </p>
          <div className="landing-hero-facts">
            <span><code>.prnc</code> and <code>.princi</code> are equivalent</span>
            <span>One language, two source suffixes</span>
          </div>
          <div className="hero-actions landing-hero-actions">
            <a
              className="button-link button-primary"
              href={repositoryUrl}
              rel="noreferrer"
              target="_blank"
            >
              View on GitHub <ArrowUpRight />
            </a>
            <ButtonLink href="/docs" variant="secondary">
              Read the docs <ArrowUpRight />
            </ButtonLink>
          </div>
        </div>

        <div className="landing-hero-example">
          <CodeSample
            code={factorialSource}
            title="Factorial and main"
            filename="factorial.prnc"
          />
          <div className="landing-output-row">
            <span>Build</span>
            <code>princi build factorial.prnc</code>
            <span aria-hidden="true" className="pipeline-arrow">→</span>
            <strong>factorial.exe</strong>
          </div>
        </div>
      </section>

      <section className="landing-section why-section">
        <div className="page-width">
          <SectionHeading
            title="Small programs stay direct. Larger ones have structure."
            description="Princi combines explicit blocks and static types with local inference and a single native build command."
          />
          <div className="why-grid">
            <article>
              <span className="why-index">SYNTAX</span>
              <h3>Concise syntax</h3>
              <p>Typed function signatures, inferred local values, and optional semicolons keep everyday code compact.</p>
            </article>
            <article>
              <span className="why-index">CHECKING</span>
              <h3>Static checks</h3>
              <p>Names, types, calls, members, and control-flow conditions are checked before native code is emitted.</p>
            </article>
            <article>
              <span className="why-index">DATA</span>
              <h3>Two data shapes</h3>
              <p>Classes share managed references; structs copy their fields by value. Both fit ordinary structured programs.</p>
            </article>
            <article>
              <span className="why-index">TARGET</span>
              <h3>Native compilation</h3>
              <p>The current compiler lowers supported programs through LLVM and links Windows x86-64 executables.</p>
            </article>
          </div>

          <Callout>
            <div className="memory-direction-grid">
              <div>
                <Badge tone="current">v0.1 today</Badge>
                <p>
                  Strings, classes, and lists use managed allocations that are
                  released when <code>main</code> returns. There is no tracing
                  collection during execution.
                </p>
              </div>
              <div>
                <Badge tone="roadmap">Longer-term direction</Badge>
                <p>
                  Progressive memory control is a language philosophy, not a
                  v0.1 capability. Ownership, borrowing, and raw-memory syntax
                  are deferred.
                </p>
              </div>
            </div>
          </Callout>
        </div>
      </section>

      <section className="page-width landing-section language-glance">
        <SectionHeading
          title="Language at a glance."
          description="These examples use syntax that the v0.1 compiler parses, checks, and lowers for its Windows target."
        />
        <div className="language-sample-grid">
          {languageSamples.map((sample) => (
            <CodeSample
              code={sample.code}
              filename={sample.filename}
              key={sample.title}
              title={sample.title}
            />
          ))}
        </div>
      </section>

      <section className="pipeline-section landing-section">
        <div className="page-width">
          <SectionHeading
            title="From source to a Windows executable."
            description="Both source extensions enter the same compiler pipeline and produce the same target format."
          />
          <ol aria-label="Princi v0.1 compilation pipeline" className="native-pipeline">
            {nativePipeline.map((stage, index) => (
              <li className="native-pipeline-step" key={stage.title}>
                <span className="pipeline-index">{String(index + 1).padStart(2, "0")}</span>
                <h3>{stage.title}</h3>
                <p>{stage.detail}</p>
              </li>
            ))}
          </ol>
          <p className="pipeline-footnote">
            Clang verifies LLVM IR and emits the Windows COFF object; MinGW-w64
            GCC links the executable and Windows C runtime.
          </p>
          <Link className="text-link text-link-large" href="/architecture">
            Compiler architecture <ArrowUpRight />
          </Link>
        </div>
      </section>

      <section className="page-width landing-section type-comparison">
        <SectionHeading
          title="Objects for identity. Structs for values."
          description="The distinction is explicit in v0.1: classes are references, while structs copy their fields."
        />
        <div className="type-comparison-grid">
          <article className="type-comparison-column">
            <div className="type-heading">
              <Badge tone="current">Class · reference</Badge>
              <h3>User</h3>
            </div>
            <p>Assigning a class value shares the same heap object. Instance methods use static dispatch.</p>
            <CodeSample code={classSource} title="Class instance" filename="user.prnc" />
            <p className="type-caption">No inheritance, virtual dispatch, or method overloading in v0.1.</p>
          </article>
          <article className="type-comparison-column">
            <div className="type-heading">
              <Badge tone="current">Struct · value</Badge>
              <h3>Point</h3>
            </div>
            <p>Assignment, parameters, and returns copy struct fields. A mutable <code>var</code> can update its fields.</p>
            <CodeSample code={structCopySource} title="Struct copy" filename="point.prnc" />
            <p className="type-caption">Structs have fields only; methods and custom layouts are not supported.</p>
          </article>
        </div>
        <Link className="text-link text-link-large" href="/language">
          Read the language guide <ArrowUpRight />
        </Link>
      </section>

      <section className="landing-section shipped-section">
        <div className="page-width">
          <SectionHeading
            title="What v0.1 actually supports."
            description="A deliberately bounded native compiler, with documented limits and no feature claims borrowed from the roadmap."
          />
          <div className="shipped-grid">
            {shippedCapabilities.map((feature) => (
              <article key={feature.title}>
                <Badge tone="current">Implemented</Badge>
                <h3>{feature.title}</h3>
                <p>{feature.description}</p>
              </article>
            ))}
          </div>
          <p className="capability-note">
            The compiler also recognizes the built-in <code>io</code> and <code>math</code> import names;
            <code>math</code> currently exports nothing. C FFI is limited to
            <code>Int32</code>, <code>Int64</code>, <code>Float64</code>, and <code>Void</code> signatures.
          </p>
        </div>
      </section>

      <section className="page-width landing-section roadmap-section">
        <SectionHeading
          title="Future direction, clearly marked."
          description="These are possible areas beyond v0.1, not features available in the compiler today. No delivery dates are promised."
        />
        <div className="roadmap-list">
          {roadmapAreas.map((area) => (
            <article key={area.title}>
              <Badge tone="roadmap">Roadmap</Badge>
              <div>
                <h3>{area.title}</h3>
                <p>{area.description}</p>
              </div>
            </article>
          ))}
        </div>
        <Link className="text-link text-link-large" href="/roadmap">
          See the full v0.1 boundary <ArrowUpRight />
        </Link>
      </section>

      <section className="page-width landing-section get-started-section">
        <div className="get-started-panel">
          <div>
            <Badge tone="current">Get started</Badge>
            <h2>One file. One build command.</h2>
            <p>
              Save the program as <code>hello.prnc</code> or <code>hello.princi</code>;
              the compiler treats them identically.
            </p>
            <ButtonLink href="/docs/installation" variant="secondary">
              Windows installation guide <ArrowUpRight />
            </ButtonLink>
          </div>
          <div className="build-command-panel" aria-label="Build command and output">
            <span>PowerShell</span>
            <code>princi build hello.prnc</code>
            <span aria-hidden="true" className="pipeline-arrow">→</span>
            <strong>hello.exe</strong>
            <small>Use <code>-o app.exe</code> to choose another output path.</small>
          </div>
        </div>
      </section>

      <section className="landing-github-cta">
        <div className="page-width landing-github-inner">
          <div>
            <h2>Open source. Built in the open.</h2>
            <p>Read the compiler, inspect the examples, and track the documented language boundary in the repository.</p>
          </div>
          <a className="button-link button-primary" href={repositoryUrl} rel="noreferrer" target="_blank">
            Explore PrinciPL on GitHub <ArrowUpRight />
          </a>
        </div>
      </section>
    </main>
  );
}
