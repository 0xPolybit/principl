import Link from "next/link";
import { pipeline, v0Features } from "@/content/princi";
import { repositoryUrl } from "@/lib/site";

const codeLines = [
  <>
    <span className="syntax-keyword">fn</span> main() &#123;
  </>,
  <>
    print(<span className="syntax-string">"Hello from Princi!"</span>)
  </>,
  <>&#125;</>,
];

export default function Home() {
  return (
    <main id="main-content">
      <section className="page-width hero">
        <div className="hero-grid">
          <div className="hero-copy">
            <h1>
              A small language.
              <br />
              A real <em>.exe.</em>
            </h1>
            <p>
              Princi is a statically typed, compiled programming language. Its
              v0.1 compiler turns focused source programs into native Windows
              x86-64 executables.
            </p>
            <div className="hero-actions">
              <Link className="button-primary" href="/installation">
                Build your first program <span aria-hidden="true">↗</span>
              </Link>
              <Link className="button-secondary" href="/language">
                Read the language guide <span aria-hidden="true">↗</span>
              </Link>
            </div>
            <div className="extension-line" aria-label="Equivalent source extensions">
              <code>.prnc</code>
              <span className="extension-equals" aria-hidden="true">
                =
              </span>
              <code>.princi</code>
              <span>same source language, same compiler</span>
            </div>
          </div>

          <section className="hero-proof" aria-label="A working Princi program">
            <div className="proof-topline">
              <span>hello.prnc</span>
              <span>Princi v0.1.0</span>
            </div>
            <pre className="proof-source" aria-label="Princi source code">
              <code>
                {codeLines.map((line, index) => (
                  <span className="proof-line" key={index}>
                    <span aria-hidden="true" className="line-number">
                      {String(index + 1).padStart(2, "0")}
                    </span>
                    {line}
                  </span>
                ))}
              </code>
            </pre>
            <div className="proof-build">
              <span className="proof-build-label">Build</span>
              <code>princi build hello.prnc</code>
            </div>
            <div className="proof-bottomline">
              <span>Windows x86-64</span>
              <strong>hello.exe</strong>
            </div>
          </section>
        </div>
      </section>

      <section aria-label="Compiler facts" className="fact-strip">
        <div className="page-width fact-strip-inner grid grid-cols-1 md:grid-cols-3">
          <div className="fact-item">
            <span aria-hidden="true" className="fact-symbol">
              .prnc
            </span>
            <p>
              <strong>One language, two suffixes</strong>
              <code>.prnc</code> and <code>.princi</code> compile identically.
            </p>
          </div>
          <div className="fact-item">
            <span aria-hidden="true" className="fact-symbol">
              → .exe
            </span>
            <p>
              <strong>Native output</strong>
              The sole v0.1 target is Windows x86-64.
            </p>
          </div>
          <div className="fact-item">
            <span aria-hidden="true" className="fact-symbol">
              Rust
            </span>
            <p>
              <strong>A compiler you can inspect</strong>
              Frontend and tooling are implemented in Rust.
            </p>
          </div>
        </div>
      </section>

      <section className="page-width section-space">
        <div className="section-header">
          <h2>Expressive foundations, with the boundary in view.</h2>
          <p>
            v0.1 brings together a typed procedural core, a small object model,
            and one built-in collection. Each feature is documented against the
            compiler that ships today.
          </p>
        </div>
        <div className="feature-list">
          {v0Features.map((feature, index) => (
            <article className="feature-item" key={feature.title}>
              <span className="pipeline-index">0{index + 1}</span>
              <h3>{feature.title}</h3>
              <p>{feature.description}</p>
              <Link href="/language">
                Language guide <span aria-hidden="true">↗</span>
              </Link>
            </article>
          ))}
        </div>
      </section>

      <section className="pipeline-section section-space">
        <div className="page-width">
          <div className="section-header">
            <h2>From source to native code.</h2>
            <p>
              The compiler checks your program before producing LLVM IR,
              Windows object code, and the executable. Intermediate files are
              handled by the compiler.
            </p>
          </div>
          <div className="pipeline-list">
            {pipeline.map((stage, index) => (
              <div className="pipeline-step" key={stage.name}>
                <span className="pipeline-index">{index + 1} / 5</span>
                <h3>{stage.name}</h3>
                <p>{stage.detail}</p>
              </div>
            ))}
          </div>
          <Link className="text-link text-link-large" href="/architecture">
            See the compiler architecture <span aria-hidden="true">↗</span>
          </Link>
        </div>
      </section>

      <section className="page-width section-space">
        <div className="build-invite">
          <div>
            <h2>Start with one file.</h2>
            <p>
              Install the Windows toolchain, write a short <code>.prnc</code> or <code>.princi</code>{" "}
              program, then use the single v0.1 build command.
            </p>
          </div>
          <div className="build-terminal">
            <div className="build-terminal-label">
              <span>PowerShell</span>
              <span>current directory</span>
            </div>
            <pre>
              <code>{"princi build hello.prnc\n.\\hello.exe"}</code>
            </pre>
            <div className="build-terminal-output">Hello from Princi!</div>
          </div>
        </div>
      </section>

      <section className="page-width boundary-line">
        <h2>Focused by design. Still early by choice.</h2>
        <div className="boundary-copy">
          <p>
            This is v0.1: useful language features and a direct native build
            path, with important platform and runtime limits. Linux and macOS,
            WebAssembly, packages, raw pointers, and advanced generics are not
            supported yet.
          </p>
          <ul>
            <li>Windows x86-64 only</li>
            <li>Built-in modules only</li>
            <li>Process-lifetime managed memory</li>
            <li>Limited C interoperability</li>
          </ul>
          <Link className="text-link text-link-large" href="/roadmap">
            Read what is deferred <span aria-hidden="true">↗</span>
          </Link>
        </div>
      </section>

      <section className="page-width boundary-line">
        <h2>Open source, from the first token.</h2>
        <div className="boundary-copy">
          <p>
            The compiler, test fixtures, examples, and v0.1 scope live in the
            project repository. Browse the implementation or follow how the
            language grows.
          </p>
          <a className="text-link text-link-large" href={repositoryUrl} rel="noreferrer" target="_blank">
            Explore PrinciPL on GitHub <span aria-hidden="true">↗</span>
          </a>
        </div>
      </section>
    </main>
  );
}
