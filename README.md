# Princi

Princi is a programming language and compiler project. Its Rust v0.1 compiler
loads, parses, and type-checks source, then generates Windows x86-64 executables
for the procedural language, classes, structs, and typed lists.

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

The compiler pipeline is:

```text
source
  → lexer
  → parser
  → AST
  → built-in module resolution
  → semantic analysis
  → typed representation
  → LLVM IR
  → Windows x86-64 COFF object
  → MinGW-w64 linker and Windows runtime
  → .exe
```

The backend lowers the typed representation to LLVM IR. Clang's LLVM IR reader
parses and verifies the module while emitting a COFF object for the explicit
x86_64-w64-windows-gnu target; only after that succeeds does MinGW-w64 GCC link
the object against the Windows C runtime. LLVM IR and object files are kept in
a temporary build directory and removed after the build. For development,
setting PRINCI_KEEP_INTERMEDIATES=1 keeps the files and reports their directory.

The compiler is implemented in Rust. The internal module boundaries are:

| Module | Responsibility |
| --- | --- |
| `cli` | Parse `princi build`, source paths, and `-o`; select the v0.1 target. |
| `diagnostics` | Report source locations and actionable compilation errors. |
| `source` | Load source text and map byte spans to file, line, and column. |
| `lexer` | Convert source text into tokens. |
| `parser` | Parse tokens into the syntax tree. |
| `ast` | Define the source-oriented abstract syntax tree. |
| `modules` | Resolve imports against the fixed v0.1 standard-module registry. |
| `ffi` | Keep C ABI types and signatures separate from Princi value types. |
| `semantic` | Check names, scopes, and language rules. |
| `types` | Represent and check source and typed-IR types. |
| `codegen` | Lower typed procedural, class, struct, and list subsets to LLVM IR, verify it, emit a Windows object, and link the executable. |
| `runtime` | Route source built-ins and managed allocation through private Windows runtime APIs. |

The CLI, source loader, diagnostics, lexer, AST, parser, built-in module
resolution, restricted C FFI, type system, semantic analysis,
procedural/class/struct/list lowering, minimal runtime, and Windows executable generation are implemented.
The v0.1 module behavior is deliberately limited to the `io` and `math`
standard-module names, described below.

## v0.1 syntax

The parser accepts top-level imports, `fn` declarations, `class` and `struct`
declarations, and `extern "C"` blocks. Functions and methods have typed
parameters, an optional `-> ReturnType`, and a block. Classes may contain typed
fields, methods, and one `init` constructor. Structs contain typed fields only;
struct methods and `init` declarations are rejected by v0.1 semantic analysis.

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
These syntax features define the v0.1 parser boundary. Native generation is
implemented for the procedural, class, and field-only struct subsets described
below; additional parsed constructs receive an explicit backend diagnostic
until their lowering exists.

## v0.1 static typing

The built-in types are `Int`, `Float`, `Bool`, `String`, and `Void`. In the
native backend, `Int` is signed 64-bit and `Float` is a 64-bit floating-point
value. Integer overflow wraps at 64 bits. Declared classes and structs
introduce nominal types with their declared names. There are no implicit
numeric conversions: arithmetic and comparisons require matching numeric
operand types. `+` also concatenates two strings; `%` accepts two
integers. Equality operators accept matching primitive types. `&&`, `||`, and
`!` require `Bool`; `if` and `while` conditions must also be `Bool`.

An omitted function or method return type means `Void`. A `Void` function may
use `return` without a value; non-`Void` returns must provide a matching value.
Function and method calls require the declared argument count and matching
argument types. The generic built-ins `print(value)` and `println(value)`
accept one non-`Void` primitive value and return `Void`; the compiler selects
the runtime implementation from the checked argument type.

`var` declares a mutable binding and `let` declares an immutable binding.
`let` requires an initializer; `var` may declare only an explicit type and
receive its value later. Parameters and `for` loop variables are immutable. A
declaration with an initializer but no annotation infers that initializer's
type, so `var x = 10` has type `Int` and `var name = "Octrie"` has type
`String`. An annotation may be used with an initializer, as in
`var count: Int = 10` or `let pi: Float = 3.14159`. Assignments must match the
binding's type, and immutable bindings cannot be assigned through.

`List<T>` is the first supported built-in generic type. List literals infer a
single element type, so `[1, 2]` has type `List<Int>`; heterogeneous literals
are rejected because v0.1 defines no common element type or numeric promotion.
An empty list needs an expected type, for example `var values: List<Int> = []`.
Bare `List` and generic arguments on other types are rejected. A range
expression requires matching numeric bounds; native `for` loops require `Int`
bounds and use an exclusive end.

Lists support `.length`, integer indexing for reads and writes, and
`.add(value)`. The compiler checks element types at each operation and
generated programs terminate with a runtime diagnostic for negative or
out-of-range indexes. Lists are heap-backed reference values: copying a list
or passing it to a function shares the same contents. `let` prevents replacing
the binding, but allows changing the list contents with indexing or `.add`.
There is no `remove`, `Map`, `Set`, `Queue`, or `Deque`, and no comprehensions,
iterators, or higher-order collection functions. Lists themselves are not
printable with `print`. Managed list storage is retained until Princi `main`
returns, then released by the runtime as a group.

Scopes are lexical. The top-level function body shares a scope with its
parameters; nested blocks and loop bodies introduce child scopes. A name may
shadow an outer local in a child scope, but duplicate declarations in the same
scope are errors. Class and struct member names share one namespace per type.
Member access checks the receiver's declared class or struct type.

The compiler/runtime use an opaque list handle with typed byte-copy accessors;
Princi source code cannot access this representation. Elements may be any
supported non-`Void` type, including nested lists, class references, and
struct values. Struct elements are copied into and out of the list by value.

Named construction (`Point { x: 1, y: 2 }`) requires every declared field
exactly once with a compatible value type and initializes those fields directly.
A positional call such as `Point(1, 2)` invokes the declared `init` constructor;
without `init`, the compiler provides a positional constructor in field
declaration order.

## Native v0.1 subset

The backend generates native code for `Int`, `Float`, `Bool`, and `String`;
local `let`/`var` bindings and assignment; arithmetic, comparison, and boolean
expressions; `if`/`else`, `while`, and integer `for` loops; functions,
parameters, returns, calls, recursion, classes, structs, instance fields,
constructors, instance methods, `List<T>`, and `print`/`println`. Lists support
homogeneous construction, local type inference, `.length`, indexed read and
write, and `.add`; the runtime checks both lower and upper index bounds. Structs
use value semantics and support fields, construction, access, mutable field
updates, function parameters, and return values; they do not support methods or
`init`.
Integer range loops use an inclusive start and exclusive end (`0..count`).
Numeric types do not implicitly convert.
`main` must take no parameters and return `Void` or `Int`; `Void` functions
return process status zero.

```princi
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

fn main() {
    let result = factorial(5)
    println(result)
}
```

This program prints `120`. Lists compile to the native v0.1 target:

```princi
fn main() {
    var numbers: List<Int> = [1, 2, 3]
    numbers[1] = 7
    numbers.add(9)
    println(numbers.length)
    println(numbers[1])
}
```

This prints `4` and `7`.

## Modules and imports

v0.1 resolves these top-level imports against a fixed compiler-provided module
registry:

```princi
import io
import math
```

Only the exact, case-sensitive names `io` and `math` are accepted. `io` names
the built-in I/O module; its `print` and `println` functions remain available
through the existing global prelude, so importing `io` is optional. `math` is
recognized as a standard module but currently exports no functions. Imports
do not create a namespace, so qualified calls such as `io.println(...)` are
not supported.

Repeated imports are idempotent. Unknown names and dotted/nested module paths
produce source-positioned diagnostics. The resolver does not search the
filesystem: user-defined modules, relative paths, project roots, and package
resolution are deferred. Since only known single-segment identifiers resolve,
there is no path traversal; the fixed registry has no candidate search or
module dependency graph, so ambiguity and import cycles cannot arise. The
`.prnc` and `.princi` suffixes share the same canonical module identities. See
[the import example](examples/modules.prnc).

## Limited C interoperability

An `extern "C"` block declares unmangled external C symbols:

```princi
extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    println(abs(-42))
}
```

v0.1 permits only `Int32`, `Int64`, and `Float64` in external parameter and
return signatures. `Void` is allowed only as a return type; omitting `->` also
means `Void`. `Int32` and `Int64` appear as Princi `Int` at call sites, and
`Float64` appears as Princi `Float`. `Int32` arguments are range-checked before
the call and `Int32` results are sign-extended. `Int64` is a signed 64-bit
integer and `Float64` is a C `double`. These ABI types are valid only inside
`extern "C"` declarations.

The backend emits LLVM's `ccc` convention for declarations and calls; on the
Windows x86-64 target this selects that target's C ABI. Calls are direct and
non-variadic, and the external symbol must be available to the normal Windows
link step (for example, through the C runtime). v0.1 does not provide extra
library/object linker flags. `Bool`, `String`, classes, structs, lists, and
other managed values cannot cross the boundary. Raw pointers, `repr(C)`
aggregates, callbacks, C++/non-C conventions, manual allocation, and pinning
remain out of scope. The type and ABI boundary lives in [`src/ffi.rs`](src/ffi.rs).

## Classes

Classes declare typed instance fields, one optional `init` constructor, and
instance methods. Use `self` to read or mutate fields from a method or
constructor. Construction with `User(args)` calls the declared initializer;
without an initializer, positional arguments initialize fields in declaration
order. Named construction such as `User { name: "Alice", age: 24 }` initializes
every field directly.

```princi
class User {
    name: String
    age: Int

    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }

    fn birthday() {
        self.age += 1
    }

    fn getAge() -> Int {
        return self.age
    }
}

fn main() {
    var user = User("Alice", 24)
    user.birthday()
    println(user.getAge())
}
```

Methods use static dispatch and are selected from the receiver's declared
class. There is no inheritance, virtual dispatch, or method overloading in
v0.1. Visibility modifiers are not implemented; fields and methods are public
within the source unit.

## Structs

Structs are field-only value types in v0.1. Fields appear in declaration order
in the LLVM aggregate layout. `Point(3.0, 4.0)` initializes fields in that
order, while named construction such as `Point { x: 3.0, y: 4.0 }` initializes
each field explicitly. Struct values are copied when assigned, passed to a
function, or returned. A `var` binding permits field mutation; `let`, parameters,
and loop bindings do not.

```princi
struct Point {
    x: Float
    y: Float
}

fn distanceSquared(point: Point) -> Float {
    return point.x * point.x + point.y * point.y
}

fn main() {
    var point = Point(3.0, 4.0)
    let snapshot = point
    point.x = 5.0
    println(distanceSquared(snapshot))
}
```

Structs have no `init`, methods, destructors, ownership syntax, or custom
memory layout attributes. A struct field whose type is a class holds a class
reference; copying the struct copies that reference, so the referenced class
instance retains identity. In contrast, class variables are references to
heap-allocated instances, and assigning one shares the same object. Class
fields/methods and struct fields are public within the source unit because
visibility modifiers are not implemented in v0.1. Recursive struct fields by
value are rejected because they have no finite layout.

## Managed memory

`String`, class instances, and `List<T>` values are managed references in the
compiler type system. Struct values are copied inline, though a struct may
contain managed references. The LLVM backend represents managed references as
opaque pointers; Princi has no raw-pointer type or manual allocation/free
operations.

In v0.1, the private managed-memory ABI uses a process-lifetime heap registry.
Dynamic String concatenation buffers, class instances, list headers, and list
backing storage are all registered through the same allocator. The generated entry
wrapper releases every registered block after Princi `main` returns; runtime
failure paths also release the managed heap before exiting. String literals remain
immutable module constants and do not use the heap.

The heap never collects individual unreachable values while a program is
running. Temporary objects, abandoned lists, concatenation results, and old
list buffers retained during growth therefore contribute to peak memory until
exit. This keeps aliases and cycles valid without source-level lifetime rules,
but memory use can grow throughout a long-running program. A future tracing
collector, potentially generational or concurrent, can replace the private
allocation ABI without changing Princi syntax. Raw pointers, `alloc<T>`,
`free`, `pin`, borrowing, ownership, and user-controlled arenas are outside
v0.1.

## Built-in runtime

`print(value)` writes an `Int`, `Float`, `Bool`, or `String` without a
trailing line break. `println(value)` writes the same primitive values followed
by a line break. The type-specific `princi_rt_*` symbols are compiler-internal;
they are not callable from Princi source.

String literals use immutable, NUL-terminated UTF-8 storage in the generated
module. String concatenation uses the managed heap described above. The
generated Windows entry wrapper calls Princi `main`, releases managed storage,
and passes its `Int` result to the MinGW C runtime as the process exit code, or
returns zero for a `Void` entry point. The runtime is embedded into generated
LLVM IR and linked automatically whenever the `princi build` command runs.

Class instances use zero-initialized heap storage. Their internal layout starts
with a type-metadata pointer, followed by fields in declaration order. Fields
and methods are public within the source unit; methods use static dispatch and
do not require vtables.

## Supported lexical syntax

The lexer recognizes identifiers beginning with a Unicode letter or `_` and
continuing with letters, digits, or `_`; decimal integer and floating-point
literals (including exponents); double-quoted strings with `\\`, `\"`, `\n`,
`\r`, `\t`, and `\0` escapes; and `true`/`false` boolean literals. It recognizes
the keywords `fn`, `return`, `let`, `var`, `if`, `else`, `while`, `for`, `in`,
`class`, `struct`, `init`, `self`, `import`, and `extern`.

Supported operators are `+`, `-`, `*`, `/`, `%`, `=`, `==`, `!=`, `<`, `<=`,
`>`, `>=`, `&&`, `||`, `!`, `+=`, `-=`, and `*=`. Punctuation includes
parentheses, braces, brackets, comma, dot, colon, semicolon, `->`, and `..`.
Whitespace and `//` line comments are skipped. The lexer preserves semicolons
when present; whether they are required at a statement boundary is a parser
rule.

Each token carries its source file, 1-based line and character column, and
end-exclusive UTF-8 byte span. Located diagnostics use the format
`file:line:column: error[Exxxx]: message` and show the relevant source span.

## Out of scope for v0.1

The following are explicitly excluded:

- Package manager, REPL, formatter, documentation generator, debugger, and IDE
  integration.
- Linux and macOS targets, WebAssembly, and targets other than Windows x86-64.
- Raw pointers, unsafe blocks, ownership and borrowing, and user-controlled arenas.
- Async/await, traits, interfaces, inheritance, advanced generics, reflection,
  and Python interoperability.

See [the v0.1 scope and architecture](docs/v0.1-scope.md) for the canonical
boundary and details.

## Installation and development

Install the Rust stable toolchain with Cargo, LLVM/Clang 15 or newer with LLVM
IR input support and the X86 backend, and MinGW-w64 GCC for the
x86_64-w64-mingw32 target. clang --print-targets lists registered targets;
gcc -dumpmachine should report x86_64-w64-mingw32. The compiler accepts
PRINCI_CLANG and PRINCI_CC to select those executables explicitly. Missing or
incompatible toolchain components produce build diagnostics.
Build or install the CLI from the repository root:

```text
cargo build --release
cargo install --path .
```

Run the automated tests with:

```text
cargo test
```

The suite includes lexer/parser/semantic/LLVM unit tests, CLI diagnostics, and
fixture-driven conformance checks under `tests/fixtures`. On Windows, native
integration tests compile generated `.exe` files, launch them, and check their
stdout, stderr, and exit status. Those tests require LLVM/Clang with X86 support
and x86-64 MinGW-w64 GCC. If either tool is unavailable, native integration
tests emit a skip note (visible with `cargo test -- --nocapture`) while the
compiler and diagnostic tests continue to run. To make a missing native
toolchain fail the test run instead of allowing those skips, set
`PRINCI_REQUIRE_NATIVE_TESTS` before running Cargo:

```powershell
$env:PRINCI_REQUIRE_NATIVE_TESTS = "1"
cargo test
```

The conformance fixtures include matching `.prnc` and `.princi` programs and
invalid programs that verify diagnostics and confirm failed builds do not
produce executables.

The build command accepts either source extension:

```text
princi build hello.prnc
princi build hello.princi
princi build hello.prnc -o program.exe
```

The CLI validates and normalizes paths, then runs the lexer, parser, semantic
analysis, LLVM IR verification, Windows x86-64 object generation, and native
linker. Failed builds exit non-zero and do not publish a partial executable.

## Diagnostics and toolchain troubleshooting

Compiler diagnostics have stable error codes. Source errors show the file,
line, column, source line, and a caret under the relevant span. For example:

```text
example.prnc:6:12: error[E0201]: expected Int, found String (expression)

    var age: Int = "twenty"
                   ^^^^^^^^^
```

Common codes include `E0002` for unsupported extensions, `E0003` for source
file errors, `E0100` for lexical errors, and `E0101` for syntax errors,
`E0201` for type mismatches, `E0202` for unknown identifiers or functions,
`E0203` for unknown types, `E0204` for incorrect argument counts, `E0205` for
invalid member access, `E0206` for duplicate declarations, `E0207` for a
missing or invalid `main`, `E0208` for invalid returns, and `E0209` for
unsupported imports.

Toolchain errors are separate from source errors: `E0401` means LLVM/Clang is
missing, `E0403` means the Windows MinGW linker is missing or targets the wrong
platform, and `E0404` means linking failed. Install LLVM/Clang with LLVM IR and
X86 support and x86-64 MinGW-w64 GCC, or point `PRINCI_CLANG` and `PRINCI_CC` at
those tools. A generated-IR failure is reported as an internal compiler error
(`E9002`); set `PRINCI_KEEP_INTERMEDIATES=1` to preserve its `.ll` input for
investigation. Unexpected compiler failures use `E9001` and exit non-zero
without exposing a Rust panic or backtrace.
