# Princi

Princi is a programming language and compiler project. This repository defines
the v0.1 scope and the compiler architecture; the source-language grammar and
construct-level semantics have not yet been specified.

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

The intended implementation language for the compiler is Rust. The internal
module boundaries are:

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

These are architecture boundaries, not a claim that compiler implementation
already exists. The current repository contains no compiler source or build
configuration. The construct-level language grammar, type rules, and runtime
surface must be specified before they can be implemented without inventing
language behavior.

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
