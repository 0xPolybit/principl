The runtime supplies private native helpers needed by generated Princi programs. `runtime.rs` is the compiler-facing facade; the Windows x86-64 implementation is in `runtime/windows_x86_64.rs` and is embedded into generated LLVM IR.

## Source built-ins

The language exposes `print(value)` and `println(value)` for primitive `Int`, `Float`, `Bool`, and `String` values. Semantic analysis checks the value type, then code generation selects a private runtime routine such as `princi_rt_print_int` or `princi_rt_println_string`. The ABI helper names are not Princi source declarations.

~~~princi
fn main() {
    print("count: ")
    println(3)
    println(true)
}
~~~

The runtime also provides string concatenation/equality, class allocation, list creation/access/growth, bounds diagnostics, checked FFI conversion, and process-startup/shutdown helpers. These routines use the Windows C runtime for basic printing and heap primitives.

## Startup and termination

The backend generates the native `main` wrapper. It calls the Princi top-level `main`, uses zero for a `Void` result, or converts an `Int` result to the process status. It then shuts down the managed heap registry. Runtime error paths such as list bounds failures also release registered managed blocks before exiting with failure.

The runtime code is part of the generated program's LLVM module and native object; users do not manually link a separate Princi runtime file.

## Platform boundary

Language-level code generation calls runtime facade functions. Platform-specific LLVM definitions live under `runtime/windows_x86_64.rs`, keeping the v0.1 Windows ABI details out of the source-language type model. The current build only emits Windows x86-64 programs.

## Current v0.1 limitations

This is a small compiler/runtime ABI, not an installable general-purpose runtime library. It does not provide a tracing collector, finalizers, user allocation/free functions, raw pointers, concurrency APIs, or portable runtime targets. See [Managed Memory](/docs/compiler/managed-memory) for the actual allocation lifetime.

## Related topics

- [Managed memory](/docs/compiler/managed-memory)
- [LLVM backend](/docs/compiler/llvm-backend)
- [Built-in functions](/docs/reference/built-in-functions)
- [Windows linking](/docs/compiler/windows-toolchain)
