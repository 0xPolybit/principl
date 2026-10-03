The compiler is implemented in Rust and moves through narrow, one-way layers:

| Module | Responsibility |
| --- | --- |
| `cli` | Parse the build command and prepare input/output options |
| `source` | Load text and map byte spans to file, line, and column |
| `diagnostics` | Format source, compiler, and toolchain errors |
| `lexer` | Convert source text to located tokens |
| `parser` / `ast` | Parse tokens into a span-preserving syntax tree |
| `modules` / `ffi` | Resolve built-in imports and validate the restricted C ABI |
| `semantic` / `types` | Resolve names and types, then produce a typed representation |
| `codegen` | Lower typed code to LLVM IR, verify it, emit an object, and link |
| `runtime` | Provide private Windows printing, allocation, list, and startup helpers |

Diagnostics and source locations are shared by frontend stages. The parser consumes located tokens and produces the AST. Module resolution records the canonical built-in modules. Semantic analysis checks the AST and retains checked types. Code generation consumes only the typed result, so semantic errors stop before native output.

## Backend boundary

The compiler sets the Windows GNU x86-64 target explicitly. Clang parses and verifies LLVM IR while emitting a COFF object. MinGW-w64 GCC links that object with the Windows C runtime to produce the final executable.

The compiler does not inherit a target accidentally from the machine hosting the process. See the [pipeline](/docs/compiler/compilation-pipeline) and [Windows toolchain](/docs/compiler/windows-toolchain) pages.
