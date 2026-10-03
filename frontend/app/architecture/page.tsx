import type { Metadata } from "next";
import { DocShell } from "@/components/doc-shell";
import { pipeline } from "@/content/princi";
import { pageTitle } from "@/lib/site";

export const metadata: Metadata = {
  title: pageTitle("Compiler architecture"),
  description:
    "Explore the Princi Rust compiler stages, LLVM IR generation, Windows object emission, runtime ABI, and MinGW-w64 link step.",
};

const modules = [
  ["cli", "Parses the single build command and prepares source/output paths."],
  ["source", "Loads source text and maps UTF-8 byte spans to line and column."],
  ["lexer · parser · ast", "Turns either source extension into the same span-preserving syntax tree."],
  ["modules · ffi", "Resolves fixed standard module names and validates the limited C ABI boundary."],
  ["semantic · types", "Checks declarations, scopes, mutability, types, calls, and members."],
  ["codegen", "Lowers the typed program to explicit Windows x86-64 LLVM IR, then object code."],
  ["runtime", "Adds private Windows print, allocation, list, and process entry helpers."],
  ["diagnostics", "Keeps source, compiler, and toolchain failures distinct and located where possible."],
];

export default function ArchitecturePage() {
  return (
    <DocShell
      description="A small set of one-way stages turns source text into a typed program, verified LLVM IR, a Windows object, and an executable."
      note="Rust · LLVM · Windows x86-64"
      title="Inside the compiler"
    >
      <section className="article-section">
        <h2>The planned pipeline, implemented for v0.1.</h2>
        <div className="flow-diagram" aria-label="Princi compiler pipeline">
          {pipeline.map((stage, index) => (
            <div className="flow-node" key={stage.name}>
              <span>STAGE {index + 1}</span>
              <strong>{stage.name}</strong>
              <p>{stage.detail}</p>
            </div>
          ))}
        </div>
        <p className="pt-6">
          The full sequence is source → lexer → parser → AST → built-in module
          resolution → semantic analysis → typed representation → LLVM IR →
          Windows COFF object → MinGW-w64 link → <code>.exe</code>.
        </p>
      </section>

      <section className="article-section">
        <h2>Compiler modules.</h2>
        <div className="module-grid">
          {modules.map(([name, description]) => (
            <div className="module-entry" key={name}>
              <code>{name}</code>
              <p>{description}</p>
            </div>
          ))}
        </div>
      </section>

      <section className="article-section">
        <h2>Windows-native backend.</h2>
        <p>
          The backend writes an explicit <code>x86_64-w64-windows-gnu</code> target triple
          and data layout. Clang&apos;s LLVM reader verifies the generated module
          while emitting a COFF object. MinGW-w64 GCC links the object with the
          Windows C runtime. The command invokes both tools with structured
          arguments, so source and output paths can contain spaces.
        </p>
        <p>
          Intermediate LLVM IR and object files live in a temporary build
          directory and are cleaned up after the build. Set
          <code>PRINCI_KEEP_INTERMEDIATES=1</code> for compiler development diagnostics.
        </p>
      </section>

      <section className="article-section">
        <h2>Runtime and memory.</h2>
        <p>
          Runtime helpers are embedded into the LLVM module and linked
          automatically. <code>print</code> and <code>println</code> resolve to type-specific private
          ABI calls. Strings, class instances, and lists use an internal
          process-lifetime heap registry; managed blocks are released when
          <code>main</code> returns or the runtime exits on an error.
        </p>
        <p>
          The registry does not collect unreachable values during program
          execution. This keeps references valid but lets allocation-heavy
          programs grow until exit. A future collector is a possibility, not a
          v0.1 capability.
        </p>
      </section>

      <section className="article-section">
        <h2>One front end for two extensions.</h2>
        <p>
          <code>.prnc</code> and <code>.princi</code> are accepted by the same source loader. The
          suffix does not change grammar, types, semantic rules, or LLVM
          generation. The compiler derives the default <code>.exe</code> name from the
          source path, unless <code>-o</code> supplies another destination.
        </p>
      </section>
    </DocShell>
  );
}
