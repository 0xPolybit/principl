Every source file follows the same compilation path:

~~~text
.prnc / .princi
  → source loading
  → lexer
  → parser
  → AST
  → built-in module resolution
  → semantic analysis
  → typed representation
  → LLVM IR
  → Windows x86-64 COFF object
  → MinGW-w64 link
  → .exe
~~~

## Frontend and typed representation

Source loading validates the input and indexes line/column locations. The lexer emits located tokens, then the parser builds an AST with spans. Module resolution accepts only built-in names. Semantic analysis validates scopes, types, calls, members, constructors, and returns. If source diagnostics exist, the backend is not run.

## LLVM and Windows native output

The backend lowers the typed representation to LLVM IR. Clang verifies and emits the object for the explicit `x86_64-w64-windows-gnu` target. MinGW-w64 GCC links the object and Windows C runtime. The resulting file is a Windows x86-64 executable.

Intermediate LLVM IR and object files are placed in a temporary build directory and normally removed. For compiler development, set `PRINCI_KEEP_INTERMEDIATES=1` to retain them.
