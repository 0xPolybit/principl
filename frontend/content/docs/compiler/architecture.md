PrinciPL's compiler is implemented in Rust. Its modules have narrow responsibilities, and compilation data moves in one direction: source text is tokenized, parsed, resolved and checked, then lowered into a Windows executable.

## Module map

| Module | Current responsibility |
| --- | --- |
| `cli` | Parse `princi build`, source path, and optional `-o` argument. |
| `compiler` | Hold `BuildOptions`, load the source, and coordinate the pipeline. |
| `source` | Read UTF-8 source and map byte spans to file, line, and column. |
| `diagnostics` | Carry error codes, locations, excerpts, and caret spans. |
| `lexer` | Convert source characters into located tokens. |
| `parser` | Consume tokens and build the source AST, recovering from syntax errors. |
| `ast` | Define declarations, statements, expressions, and source spans. |
| `modules` | Resolve imports against the built-in `io` and `math` registry. |
| `semantic` | Build symbol scopes, resolve names, check language rules, and retain types. |
| `types` | Represent source types and compiler-internal function/range/error types. |
| `ffi` | Represent the small C ABI signature separately from Princi types. |
| `codegen` | Generate LLVM IR and orchestrate native object and executable creation. |
| `runtime` | Provide private built-in, allocation, list, and process-startup helpers. |

The repository also has implementation submodules: `compiler/options.rs` defines `CompilerOptions` and `BuildOptions`; `compiler/pipeline.rs` sequences the stages; `codegen/llvm.rs` contains the LLVM lowering and Windows toolchain bridge; `runtime/windows_x86_64.rs` contains target-specific LLVM runtime definitions. `main.rs` maps errors to process exit codes and converts an unexpected Rust panic to an internal diagnostic.

## Data flow

`Compiler::build` receives normalized build options and asks `SourceFile` to load the source. The pipeline then lexes, parses, resolves imports, checks semantics, produces a typed representation, emits LLVM IR, and invokes the native toolchain. Syntax, module, and semantic failures stop compilation before code generation; backend and linker failures return their own diagnostics.

The semantic result contains a `TypedProgram`: the AST, resolved symbols and imports, and type maps for expressions, variables, and parameters. Code generation consumes that checked result. It does not ask the CLI to make language decisions.

## Current target boundary

The compiler has one v0.1 target: Windows x86-64 using the GNU toolchain triple. The backend emits COFF through LLVM/Clang and MinGW-w64 GCC links the executable. `.prnc` and `.princi` reach the same compiler path. See the [compilation pipeline](/docs/compiler/compilation-pipeline), [LLVM backend](/docs/compiler/llvm-backend), and [Windows linking](/docs/compiler/windows-toolchain).

## Current v0.1 limitations

This module map describes the current compiler, not planned subsystems. Imports resolve only built-in names; the runtime uses process-lifetime managed storage rather than a tracing collector; and native output targets Windows x86-64 only. There is no package manager, REPL, alternate target backend, debugger, or IDE integration.

## Related topics

- [Compilation pipeline](/docs/compiler/compilation-pipeline)
- [Lexer & Parser](/docs/compiler/lexer-parser)
- [AST](/docs/compiler/ast)
- [Semantic analysis](/docs/compiler/semantic-analysis)
- [Managed memory](/docs/compiler/managed-memory)
