import type { Metadata } from "next";
import { CodeSample } from "@/components/code-sample";
import { DocShell } from "@/components/doc-shell";
import { Callout } from "@/components/site-primitives";
import { helloSource } from "@/content/princi";
import { pageTitle } from "@/lib/site";

export const metadata: Metadata = {
  title: pageTitle("Install Princi"),
  description:
    "Install the Windows x86-64 Princi toolchain, build the Rust compiler, and compile a first .prnc or .princi program.",
};

const prerequisites = [
  {
    title: "Rust stable and Cargo",
    detail: "Used to build or install the compiler from the repository.",
    command: "rustc --version",
  },
  {
    title: "LLVM/Clang 15 or newer",
    detail: "Clang must accept LLVM IR and include the X86 target backend.",
    command: "clang --print-targets",
  },
  {
    title: "x86-64 MinGW-w64 GCC",
    detail: "The linker target must be x86_64-w64-mingw32.",
    command: "gcc -dumpmachine",
  },
];

export default function InstallationPage() {
  return (
    <DocShell
      description="Build the compiler from a clean clone, then use its single v0.1 command to turn a source file into a Windows executable."
      note="Windows x86-64"
      title="Install and build"
    >
      <section className="article-section">
        <h2>Install the native toolchain.</h2>
        <p>
          The compiler itself is Rust. Producing a Windows executable also
          needs LLVM/Clang to verify and emit the Windows object file, then
          MinGW-w64 GCC to link that object.
        </p>
        <ol className="steps-list">
          {prerequisites.map((item) => (
            <li key={item.title}>
              <div>
                <strong>{item.title}</strong>
                <p>{item.detail}</p>
                <p>
                  Check it with <code>{item.command}</code>.
                </p>
              </div>
            </li>
          ))}
        </ol>
        <Callout>
          <strong>Windows setup</strong>
          <p>
            The repository README includes a PowerShell and MSYS2 UCRT64 setup
            walkthrough, including the expected GCC target triple.
          </p>
        </Callout>
      </section>

      <section className="article-section">
        <h2>Build the compiler.</h2>
        <p>
          Clone the compiler repository and install its release executable in
          Cargo&apos;s binary directory.
        </p>
        <CodeSample
          code={`git clone https://github.com/0xPolybit/principl.git
cd principl
cargo install --path .`}
          language="powershell"
          title="PowerShell"
          filename="compiler setup"
        />
        <p>
          Cargo installs <code>princi</code> into <code>%USERPROFILE%\.cargo\bin</code>. If the
          command is not found, add that directory to <code>PATH</code> and open a new
          terminal. You can also run the local debug compiler from the clone
          with <code>cargo run -- build hello.prnc</code>.
        </p>
      </section>

      <section className="article-section">
        <h2>Compile a first program.</h2>
        <p>
          Save this exact source as <code>hello.prnc</code> or <code>hello.princi</code>. Both
          extensions compile as the same language.
        </p>
        <CodeSample
          code={helloSource}
          title="Hello from Princi"
          filename="hello.prnc or hello.princi"
        />
        <CodeSample
          code={`princi build hello.prnc
.\\hello.exe`}
          language="powershell"
          title="Build and run"
          filename="PowerShell"
        />
        <Callout>
          <strong>Expected output</strong>
          <p>
            The executable prints <code>Hello from Princi!</code> without a
            trailing line break. By default it is named{" "}<code>hello.exe</code> beside the
            source file.
          </p>
        </Callout>
      </section>

      <section className="article-section">
        <h2>Choose another output path.</h2>
        <p>
          <code>-o</code> selects the executable path. Relative output paths are resolved
          from the current working directory.
        </p>
        <CodeSample
          code="princi build hello.prnc -o app.exe"
          language="powershell"
          title="Explicit output"
          filename="PowerShell"
        />
        <p>The command writes <code>app.exe</code>.</p>
      </section>

      <section className="article-section">
        <h2>Run the compiler checks.</h2>
        <p>
          From the repository root, <code>cargo test</code> runs compiler unit tests, CLI
          diagnostics, and fixture conformance. On Windows, native integration
          checks build and launch executables when LLVM/Clang and MinGW-w64 GCC
          are available.
        </p>
        <p>
          To make a missing native toolchain fail rather than skip those checks,
          set <code>PRINCI_REQUIRE_NATIVE_TESTS=1</code> before running <code>cargo test</code>.
        </p>
      </section>
    </DocShell>
  );
}
