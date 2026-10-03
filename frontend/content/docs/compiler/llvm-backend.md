The v0.1 backend consumes a `TypedProgram` and lowers the supported language subset into LLVM IR. It is implemented in `codegen/llvm.rs`; the public `codegen` module forwards the IR generation and native build operations.

## Target-aware IR

The generated module declares both the Windows GNU x86-64 target triple (`x86_64-w64-windows-gnu`) and its data layout. The backend defines LLVM representations for primitive values, class layouts, and inline struct aggregates. Class instances and lists use opaque runtime-managed pointers; strings are pointers to NUL-terminated UTF-8 storage.

The lowering handles functions, parameters, calls and recursion; local bindings and assignment; arithmetic, comparisons and short-circuit boolean operations; branches, loops and returns; class fields, constructors and static methods; struct construction and value copies; list operations; and primitive printing. Runtime helper definitions are included in the module.

~~~text
typed program → target-aware LLVM IR module
             → Windows entry wrapper + runtime helpers
~~~

The backend checks that `main` exists, takes no parameters, and returns `Void` or `Int`. It generates the Windows entry wrapper and releases managed allocations when source `main` returns.

## Verification and object generation

LLVM IR is written into the temporary build directory. Clang reads the IR, checks it while emitting code, and produces a Windows x86-64 COFF object. The backend does not use an accidental host default: the target is explicit. The later link step is described in [Windows Linking](/docs/compiler/windows-toolchain).

IR generation failures are distinguished from a missing Clang executable and native object-emission failures. See [Diagnostics](/docs/compiler/diagnostics).

## Current v0.1 limitations

The backend implements the procedural, class, struct, list, and runtime features enumerated in the current v0.1 scope. Unsupported AST/type combinations receive a compiler diagnostic instead of silently emitting a substitute. LLVM IR and object files are normally cleaned up after the build; `PRINCI_KEEP_INTERMEDIATES=1` keeps them for development.

## Related topics

- [Type system](/docs/compiler/type-system)
- [Compilation pipeline](/docs/compiler/compilation-pipeline)
- [Windows linking](/docs/compiler/windows-toolchain)
- [Runtime](/docs/compiler/runtime)
