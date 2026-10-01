# Princi

Princi is a programming language and compiler project. The v0.1 compiler CLI is
implemented in Rust. The source-language grammar and construct-level semantics
are still being specified, so compilation currently stops with a diagnostic at
the lexer stage.

## v0.1 compiler contract

The only user-facing compiler workflow in v0.1 is:

```text
princi build <source-file>
```

Princi source files may use either `.prnc` or `.princi`. The extensions are
interchangeable: both identify the same source language and must have identical
parsing, semantics, and compilation behavior. For example, either of these
commands produces `hello.exe` beside its source file:

```text
princi build hello.prnc
princi build hello.princi
```

An output path can be selected with `-o`:

```text
princi build hello.prnc -o app.exe
```

The sole v0.1 target is Windows x86-64. There are no other user-facing compiler
commands in this version.

## Compiler architecture

The planned compilation pipeline is:

```text
source
  → lexer
  → parser
  → AST
  → semantic analysis
  → typed representation
  → LLVM IR
  → Windows object/native code
  → .exe
```

The compiler is implemented in Rust. The internal module boundaries are:

| Module | Responsibility |
| --- | --- |
| `cli` | Parse `princi build`, source paths, and `-o`; select the v0.1 target. |
| `diagnostics` | Report source locations and actionable compilation errors. |
| `lexer` | Convert source text into tokens. |
| `parser` | Parse tokens into the syntax tree. |
| `ast` | Define the source-oriented abstract syntax tree. |
| `semantic` | Check names, scopes, and language rules. |
| `types` | Represent and check source and typed-IR types. |
| `codegen` | Lower typed representation through LLVM IR to Windows output. |
| `runtime` | Provide only runtime support required by v0.1 programs. |

The CLI and build-option layer are implemented. The remaining compiler modules
will be filled in as the source grammar, type rules, entry-point convention, and
runtime surface are specified.

## Out of scope for v0.1

The following are explicitly excluded:

- Package manager, REPL, formatter, documentation generator, debugger, and IDE
  integration.
- Linux and macOS targets, WebAssembly, and targets other than Windows x86-64.
- Raw pointers, unsafe blocks, ownership and borrowing, and arenas.
- Async/await, traits, interfaces, inheritance, advanced generics, reflection,
  and Python interoperability.

See [the v0.1 scope and architecture](docs/v0.1-scope.md) for the canonical
boundary and details.

## Installation and development

Install the Rust stable toolchain with Cargo, then build or install the CLI from
the repository root:

```text
cargo build --release
cargo install --path .
```

Run the automated tests with:

```text
cargo test
```

The build command accepts either source extension:

```text
princi build hello.prnc
princi build hello.princi
princi build hello.prnc -o program.exe
```

The CLI validates and normalizes the input and output paths, then enters the
compiler pipeline. Until the lexer and later compilation stages are implemented,
valid build commands report that stage limitation and exit non-zero; they do not
claim to have produced an executable.
