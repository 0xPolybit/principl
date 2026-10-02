# Princi

Princi is a programming language and compiler project. Its v0.1 Rust frontend
includes source loading, located diagnostics, a lexer, an AST, type checking,
and semantic analysis. Native executable generation is still under development.

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

The CLI, source loader, diagnostics, lexer, AST, parser, type system, semantic
analysis, compiler options, and pipeline entry point are implemented. Typed IR,
LLVM lowering, Windows executable generation, and runtime support remain under
development.

## v0.1 syntax

The parser accepts top-level `import` paths, `fn` declarations, `class`
declarations, and `struct` declarations. Functions and methods have typed
parameters, an optional `-> ReturnType`, and a block. Classes and structs may
contain typed fields, methods, and `init` constructors.

Function bodies support `let` and `var` declarations (inferred or explicitly
typed), assignments, nested blocks, `if`/`else`, `while`, `for name in
start..end`, `return`, and expression statements. Expressions include
identifiers, `self`, integer/floating-point/string/boolean literals, list
literals, `Type { field: value }` construction, calls, member access, indexing,
parentheses, unary `+`/`-`/`!`, ranges, and the binary operators listed in the
lexical syntax section.

Semicolons may separate statements; a line break also separates statements
when the next token cannot continue the preceding expression. Newlines after an
operator continue the expression. The range operator is non-associative unless
parenthesized. Binary operators associate left-to-right. From low to high, the
precedence is `..`, `||`, `&&`, `==`/`!=`, comparisons, `+`/`-`, `*`/`/`/`%`,
unary operators, then calls, member access, and indexing.

The parser retains end-exclusive UTF-8 byte spans throughout the AST and
reports source-positioned syntax diagnostics. It recovers at statement,
member, and declaration boundaries so a build can report multiple parse errors.
These syntax features define the v0.1 parser boundary; they do not imply that
native executable generation is complete.

## v0.1 static typing

The built-in types are `Int`, `Float`, `Bool`, `String`, and `Void`. Declared
classes and structs introduce nominal types with their declared names. There
are no implicit numeric conversions: arithmetic and comparisons require matching
numeric operand types. `+` also concatenates two strings; `%` accepts two
integers. Equality operators accept matching primitive types. `&&`, `||`, and
`!` require `Bool`; `if` and `while` conditions must also be `Bool`.

An omitted function or method return type means `Void`. A `Void` function may
use `return` without a value; non-`Void` returns must provide a matching value.
Function and method calls require the declared argument count and matching
argument types. `print(value)` is a built-in that accepts one non-`Void` value
and returns `Void`.

`var` declares a mutable binding and `let` declares an immutable binding.
`let` requires an initializer; `var` may declare only an explicit type and
receive its value later. Parameters and `for` loop variables are immutable. A
declaration with an initializer but no annotation infers that initializer's
type, so `var x = 10` has type `Int` and `var name = "Octrie"` has type
`String`. An annotation may be used with an initializer, as in
`var count: Int = 10` or `let pi: Float = 3.14159`. Assignments must match the
binding's type, and immutable bindings cannot be assigned through.

List literals infer a single element type: `[1, 2]` has internal type
`List<Int>`. Mixed element types are rejected. An empty list needs an expected
element type; the v0.1 source annotation `List` represents `List<Any>` and can
be used for empty or dynamically typed lists. Generic source syntax such as
`List<Int>` is not part of v0.1. Indexing requires an `Int` and returns the
list's element type. A range requires matching numeric bounds and a `for` loop
binds its variable to that numeric type.

Scopes are lexical. The top-level function body shares a scope with its
parameters; nested blocks and loop bodies introduce child scopes. A name may
shadow an outer local in a child scope, but duplicate declarations in the same
scope are errors. Class and struct member names share one namespace per type.
Member access checks the receiver's declared class or struct type.

Named construction (`Point { x: 1, y: 2 }`) requires every declared field
exactly once with a compatible value type and initializes those fields directly.
A positional call such as `Point(1, 2)` invokes the declared `init` constructor;
without `init`, the compiler provides a positional constructor in field
declaration order.

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

The CLI validates and normalizes the input and output paths, then runs the
lexer, parser, and semantic analysis. Semantic errors stop the pipeline with
source-positioned diagnostics. Until native code generation is implemented,
semantically valid programs receive an explicit code-generation-incomplete
diagnostic and the command exits non-zero without claiming to have produced an
executable.
