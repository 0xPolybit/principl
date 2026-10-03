Every build uses the same ordered stages for `.prnc` and `.princi`. The diagram shows the actual v0.1 path implemented by `compiler::pipeline` and the native backend.

<!-- princi-diagram:compiler-pipeline -->

## Frontend stages

1. **Source loading.** The CLI resolves the source path and derives or normalizes the output path. `SourceFile` reads UTF-8 text and indexes line starts for diagnostics.
2. **Lexer.** Source characters become tokens with source locations. Malformed literals and unexpected characters produce lexical diagnostics.
3. **Parser and AST.** The parser consumes tokens, builds the source-oriented AST, and recovers at syntax boundaries to report errors. A build stops if parser diagnostics were collected.
4. **Module resolution.** Imports are checked against the fixed, case-sensitive standard registry (`io`, `math`). Resolution does not search files on disk.
5. **Semantic analysis.** Declarations, scopes, names, calls, assignments, control flow, and types are checked. Success returns a typed representation. Any source errors stop the backend.

## Native stages

6. **LLVM IR.** `codegen::llvm` lowers the typed program and includes the explicit `x86_64-w64-windows-gnu` target configuration and private runtime definitions. It also validates the entry point.
7. **Windows object.** Clang reads the generated LLVM IR, verifies it as it emits code, and writes a Windows x86-64 COFF object.
8. **Executable link.** The compiler checks that the selected GCC targets `x86_64-w64-mingw32`, then asks MinGW-w64 GCC to link the object and Windows C runtime into an `.exe`.

LLVM IR and object files live in a temporary build directory and are removed after success or failure. Set `PRINCI_KEEP_INTERMEDIATES=1` to retain those compiler-development artifacts. The command remains `princi build <source-file>`; no intermediate path is required from the user.

## Failure boundaries

Source and semantic errors are reported before LLVM is generated. LLVM/Clang availability, LLVM/native emission, linker availability, and link failure have separate diagnostic codes. Failed builds return a nonzero process status and do not count as successful executable generation.

## Related topics

- [Compiler overview](/docs/compiler/architecture)
- [Lexer & Parser](/docs/compiler/lexer-parser)
- [Semantic analysis](/docs/compiler/semantic-analysis)
- [LLVM backend](/docs/compiler/llvm-backend)
- [Windows linking](/docs/compiler/windows-toolchain)
- [Diagnostics](/docs/compiler/diagnostics)
