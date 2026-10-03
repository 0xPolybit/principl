PrinciPL (usually called Princi) is a statically typed programming language and compiler project. Its v0.1 compiler is written in Rust, uses LLVM for native code generation, and targets Windows x86-64.

The language combines a compact procedural core with functions, control flow, classes, value-oriented structs, strings, and the built-in `List<T>` collection. The compiler checks types before generating a native Windows executable.

> **v0.1 status:** This describes the implementation in this repository today. Linux, macOS, WebAssembly, package management, async, traits, ownership, and raw pointers are not supported targets or language features in v0.1.

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

## What v0.1 can compile

The implemented native subset includes `Int`, `Float`, `Bool`, `String`, typed functions and recursion, local `let`/`var` bindings, arithmetic and boolean expressions, conditionals, loops, classes, field-only structs, `List<T>`, and `print`/`println`. The compiler resolves only its built-in `io` and `math` module names and supports a restricted primitive C ABI declaration syntax.

The compiler does not yet implement the broader systems-language features listed on the [v0.1 limitations](/docs/reference/limitations) page. See the repository's [full v0.1 scope](https://github.com/0xPolybit/principl/blob/main/docs/v0.1-scope.md) for precise boundaries.

## Continue

- [Getting started overview](/docs/getting-started)
- [Install Rust, LLVM/Clang, and MinGW-w64](/docs/installation)
- [Build and run Hello, world](/docs/hello-world)
- [Compile programs and select output paths](/docs/compiling)
- [Read the language syntax guide](/docs/language/syntax)
