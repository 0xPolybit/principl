# Princi

Princi is a programming language and compiler project. The v0.1 compiler CLI,
source loader, diagnostics, and lexer are implemented in Rust. The parser and
later stages are still being developed.

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
| `source` | Load source text and map byte spans to file, line, and column. |
| `lexer` | Convert source text into tokens. |
| `parser` | Parse tokens into the syntax tree. |
| `ast` | Define the source-oriented abstract syntax tree. |
| `semantic` | Check names, scopes, and language rules. |
| `types` | Represent and check source and typed-IR types. |
| `codegen` | Lower typed representation through LLVM IR to Windows output. |
| `runtime` | Provide only runtime support required by v0.1 programs. |

The CLI, source loader, diagnostics, lexer, compiler options, and pipeline entry
point are implemented. The remaining compiler modules will be filled in as the
source grammar, type rules, entry-point convention, and runtime surface are
specified.

## Supported lexical syntax

The lexer recognizes identifiers beginning with a Unicode letter or `_` and
continuing with letters, digits, or `_`; decimal integer and floating-point
literals (including exponents); double-quoted strings with `\\`, `\"`, `\n`,
`\r`, `\t`, and `\0` escapes; and `true`/`false` boolean literals. It recognizes
the keywords `fn`, `return`, `let`, `var`, `if`, `else`, `while`, `for`, `in`,
`class`, `struct`, `init`, `self`, and `import`.

Supported operators are `+`, `-`, `*`, `/`, `%`, `=`, `==`, `!=`, `<`, `<=`,
`>`, `>=`, `&&`, `||`, `!`, `+=`, `-=`, and `*=`. Punctuation includes
parentheses, braces, brackets, comma, dot, colon, semicolon, `->`, and `..`.
Whitespace and `//` line comments are skipped. The lexer preserves semicolons
when present; whether they are required at a statement boundary is a parser
rule.

Each token carries its source file, 1-based line and character column, and
end-exclusive UTF-8 byte span. Located diagnostics use the format
`file:line:column: error: message`.

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

The CLI validates and normalizes the input and output paths, lexes the source,
then reports that the parser stage is incomplete. It exits non-zero and does not
claim to have produced an executable until the remaining compilation stages are
implemented.
