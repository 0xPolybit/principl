PrinciPL (usually called Princi) is a statically typed programming language and compiler project. The v0.2 compiler is written in Rust, uses LLVM for native code generation, and targets Windows x86-64.

The language combines a compact procedural core with generics, classes, value-oriented structs, interfaces, traits, algebraic enums, pattern matching, `Result<T, E>`, and `List<T>`. The compiler checks types before generating a native Windows executable.

> **v0.2 status:** This describes the implementation in this repository today. The compiler targets Windows x86-64 only. Linux, macOS, WebAssembly, package management, async, inheritance, ownership, and raw pointers are not supported in v0.2.

## One language, two extensions

Princi source files may end in `.prnc` or `.princi`. These suffixes are exact aliases: both are loaded, parsed, type-checked, and compiled as the same language. Neither extension enables a different syntax or behavior.

## Build a Windows executable

The CLI has one user-facing command, `build`:

```powershell
princi build hello.prnc
```

This writes `hello.exe` beside `hello.prnc`. An input named `hello.princi` produces the same `hello.exe` output. Use `-o` to select a different path:

```powershell
princi build hello.prnc -o app.exe
```

The build requires Windows x86-64 native tooling. Start with [Getting started](/docs/getting-started) for the setup sequence, then follow the detailed [Installation guide](/docs/installation) and [Hello, world](/docs/hello-world) walkthrough.

## What v0.2 can compile

The compiler supports `Int`, `Float`, `Bool`, `String`, and `Void`; local `let`/`var`; functions, recursion, control flow, strings, generic functions/classes/structs/enums, generic bounds, interfaces with dynamic dispatch, traits with defaults, enum matching with exhaustiveness checks, `Result<T, E>` and `?`, lists, built-in imports, and a restricted primitive C ABI.

The [v0.2 limitations](/docs/reference/limitations) page describes the current boundary. The repository's [canonical v0.2 scope](https://github.com/0xPolybit/principl/blob/main/docs/v0.2-scope.md) records precise semantics and compatibility expectations.

## Continue

- [Getting started overview](/docs/getting-started)
- [Install Rust, LLVM/Clang, and MinGW-w64](/docs/installation)
- [Build and run Hello, world](/docs/hello-world)
- [Compile programs and select output paths](/docs/compiling)
- [Read the language syntax guide](/docs/language/syntax)
