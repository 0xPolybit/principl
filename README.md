# Princi

Princi v0.1.0 is a statically typed, compiled programming language for native
Windows x86-64 programs. The v0.1 compiler is written in Rust, lowers checked
source to LLVM IR, and links a Windows `.exe`. Its language includes procedural
programming, classes, value-oriented structs, and the built-in `List<T>` type.

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
receive its value later. A local cannot be read until it is definitely
initialized. Assignments in both arms of an `if` establish initialization;
assignments inside `while` or `for` bodies do not, because those loops may run
zero times. Parameters and `for` loop variables are immutable. A declaration
with an initializer but no annotation infers that initializer's
type, so `var x = 10` has type `Int` and `var name = "Octrie"` has type
`String`. An annotation may be used with an initializer, as in
`var count: Int = 10` or `let pi: Float = 3.14159`. Assignments must match the
binding's type, and immutable bindings cannot be assigned through.

`List<T>` is the first supported built-in generic type. List literals infer a
single element type, so `[1, 2]` has type `List<Int>`; heterogeneous literals
are rejected because v0.1 defines no common element type or numeric promotion.
An empty list needs an expected type, for example `var values: List<Int> = []`.
Bare `List` is rejected. At the v0.1 release boundary, `List<T>` is the only
generic type with native support. Generic user-defined types are part of the
active v0.2 development work described below. A range expression requires
matching numeric bounds; native `for` loops require `Int` bounds and use an
exclusive end.

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
module. Embedded NUL bytes are rejected in v0.1 so printing, concatenation,
and equality preserve the full String value. String concatenation uses the
managed heap described above. The generated Windows entry wrapper calls
Princi `main`, releases managed storage,
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
`\r`, and `\t` escapes; and `true`/`false` boolean literals. NUL bytes and
`\0` escapes are rejected because v0.1 Strings use NUL-terminated storage.
The lexer recognizes the v0.1 keywords `fn`, `return`, `let`, `var`, `if`,
`else`, `while`, `for`, `in`, `class`, `struct`, `init`, `self`, `import`, and
`extern`. v0.2 development adds `interface`, `trait`, and `implements` for
behavioral contracts and their implementations. The word `match` is parsed
contextually so existing identifiers with that name remain usable. The match
arm separator `=>` is also recognized.

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
- Raw pointers, unsafe blocks, ownership and borrowing, and user-controlled
  arenas.
- Enums, `match`, decorators, async/await, traits, interfaces, inheritance,
  virtual or abstract classes, method/operator overloading, and advanced
  generics.
- Raw C pointers, `repr(C)` structs, callbacks, variadic FFI, reflection, and
  Python interoperability.
- `Map`, `Set`, `Queue`, `Deque`, comprehensions, iterators, and higher-order
  collection functions.

See [the v0.1 scope and architecture](docs/v0.1-scope.md) for the canonical
boundary and details.

## Roadmap: proposed v0.2

The release line remains v0.1.0. The repository's v0.2 development work now
includes native monomorphization for generic functions. Explicit type arguments
and local inference from function arguments are supported when each type
parameter has one unambiguous concrete type. A specialization is emitted only
when used, reused across identical calls, and named with a deterministic
internal symbol. Same-type generic recursion is supported; recursively
expanding specialization patterns and programs requiring more than 512
specializations produce diagnostics. Inference uses call argument types only;
if those arguments do not determine every type parameter, the call must give
explicit type arguments. The compiler does not infer generic arguments from the
expected result type.

For example, both calls below use the same `Int` specialization, while the
`String` call creates a second one:

```princi
fn identity<T>(value: T) -> T {
    return value
}

fn main() {
    let number = identity<Int>(42)
    let inferred = identity(7)
    let text = identity<String>("hello")
    println(number)
    println(inferred)
    println(text)
}
```

Generic function specialization now works with concrete generic class and
struct values, including nested `List<T>` and user-defined generic types.
Generic classes and structs are specialized only for concrete instances used
by the program. Each generic class instance has its own deterministic native
layout and remains a managed reference; generic structs have specialized inline
LLVM layouts and retain value-copy behavior. Constructors accept explicit
arguments such as `Box<Int>(42)`. The compiler also infers class or struct type
arguments from constructor argument types when every parameter is determined,
as in `Box(42)`. It does not infer from an expected result type; ambiguous or
unused type parameters require explicit arguments.

The built-in `List<T>` uses this same generic instance type representation and
can contain concrete generic values such as `List<Box<Int>>` and
`List<Pair<String, Float>>`. Its runtime remains a homogeneous, size-aware byte
buffer: class elements are copied as managed references, while struct elements
are copied by value. List literals and operations retain compile-time element
checking and runtime bounds checks.

Generic methods may use their enclosing class parameters, as in
`fn get() -> T`. Method-level generic parameters are not yet lowered in this
work. Traits and runtime trait values are now implemented in the v0.2 branch
work described below. Bounded generics for functions, classes, and structs are
implemented. The v0.2 work also supports algebraic enum declarations,
generic enum types, variant construction, and by-value movement through local
variables, function parameters, and returns. Enum pattern matching and
exhaustiveness checking, the prelude `Result<T, E>`, and postfix `?`
propagation are implemented as described below.

Enums use named variants with optional positional payloads:

```princi
enum Direction {
    North
    South
    East
    West
}

enum NetworkState {
    Connected
    Connecting
    Disconnected
    Failed(String)
}

enum Maybe<T> {
    Some(T)
    None
}

fn divide(a: Int, b: Int) -> Result<Int, String> {
    if b == 0 {
        return Result<Int, String>.Err("division by zero")
    }
    return Result<Int, String>.Ok(a / b)
}

fn main() {
    let direction = Direction.North
    let failure = NetworkState.Failed("Connection refused")
    let result = divide(10, 2)
    match result {
        Ok(value) => println(value)
        Err(error) => println(error)
    }
}
```

Each native enum value has an `i32` discriminant and typed storage for each
variant payload. The current LLVM layout is an internal tagged aggregate, not a
C ABI layout. Enum values copy inline struct/enum payloads by value; String,
List, and class payloads retain their existing managed-reference behavior.
`match` inspects the active tag and extracts payloads. `Maybe<T>` is an ordinary
user-declared enum. `Result<T, E>` is included from the source prelude as the
ordinary generic enum `Ok(T) | Err(E)` and uses the regular enum type checking
and layout. Postfix `?` is compiler syntax that branches on those variants; it
does not add a special runtime representation. A top-level declaration named
`Result` replaces the prelude definition for that source file.

Construct enum values with a qualified generic type when its arguments are
not otherwise fixed by construction, for example
`Result<Int, String>.Err("division by zero")`. Match arms use the matched
value's type, so their variant patterns are written concisely as `Ok(value)`
and `Err(error)`. The prelude is embedded in the compiler and requires no
manual import or installed source file.

Postfix `?` propagates a failed result from functions that return a compatible
`Result`:

```princi
fn readNumber() -> Result<Int, String> {
    return Result<Int, String>.Ok(42)
}

fn loadNumber() -> Result<Int, String> {
    let number = readNumber()?
    return Result<Int, String>.Ok(number)
}

fn passNumber() -> Result<Int, String> {
    return readNumber()?
}
```

`operation()?` evaluates the operation once, yields the `Ok` payload as the
expression value, and immediately returns an `Err` with the same error payload
from the enclosing function. The operand and enclosing return type must use the
ordinary `Result<T, E>` enum, and their error types must match exactly; no
`From`/`Into`-style error conversion is performed. A function using `?` must
return `Result`. When the entire expression in a `return` statement is
`operation()?` (parentheses are allowed), the success payload must match the
function's `Result` success type and is wrapped in `Ok` for that return.
Elsewhere, `?` yields the success payload normally, so it can appear in local
initializers, arguments, and larger expressions. Generic functions may use
`?` when their error types match under the specialization.

The operator rejects non-`Result` operands, non-`Result` enclosing functions,
and mismatched error types at compile time.
The compiler remains Windows x86-64 only with the existing
`princi build` workflow and `.prnc`/`.princi` extension equivalence.

### Match expressions and patterns (v0.2 development)

`match` can be used as a statement or as an expression. Enum variants may be
matched by name, and positional payloads may bind immutable names or use
wildcards. Bare names matching nullary variants select them; a payload variant
must use parentheses even when its payload pattern is `_`. Other lowercase
names bind the whole matched value. A misspelled uppercase enum variant is
reported as an unknown variant.

```princi
enum NetworkState {
    Connected
    Connecting
    Disconnected
    Failed(String)
}

fn main() {
    let state = NetworkState.Failed("Connection refused")
    match state {
        Connected => println("online")
        Connecting => println("connecting")
        Failed(reason) => println(reason)
        Disconnected => println("offline")
    }

    let message = match state {
        Connected => "online"
        Connecting => "connecting"
        Failed(reason) => reason
        Disconnected => "offline"
    }
    println(message)
}
```

Every enum variant must be covered, or the match must include `_` or a binding
pattern. A non-exhaustive enum match names the missing case when possible, for
example `missing variant: Boolish.No`. Coverage is checked against the enum's
declared variants, so `Result<Int, String>` and `Result<String, Int>` use the
same variant set regardless of their payload types. `Bool` matches are
exhaustive with both `true` and `false`; other primitive matches need a
catch-all arm. Integer, floating-point, boolean, and string literals are
supported as patterns. Expression arms must have one compatible result type.
A block arm has type `Void`. Each arm has its own lexical scope, and its
bindings are immutable and unavailable after the arm. Duplicate
variants/literals and patterns after a catch-all are diagnosed as unreachable.
When nested payload patterns do not cover every payload value, the checker
conservatively requires an additional catch-all arm.

Enum matches lower to LLVM control flow using a tag switch for direct variant
patterns. Nested payload checks and primitive/string literals use additional
comparisons. Array/list patterns, struct destructuring, range patterns, and
guards are outside this v0.2 subset.

### Interfaces (v0.2)

Interfaces declare instance method signatures. They cannot declare fields,
constructors, method bodies, or generic methods. A class may implement multiple
interfaces and must provide each required method with the same parameter and
return types. Structs cannot implement interfaces, and interfaces are not a
class inheritance mechanism.

```princi
interface Drawable {
    fn draw(prefix: String) -> String
}

class Button implements Drawable {
    fn draw(prefix: String) -> String {
        return prefix + "button"
    }
}

fn render(value: Drawable) {
    println(value.draw("drawing: "))
}

fn main() {
    let button = Button()
    let drawable: Drawable = button
    render(drawable)
}
```

Conforming class instances convert to interface values in locals, function
arguments and returns, class or struct fields, and `List<Interface>` values.
The generated value is a pair containing the managed object reference and a
compiler-generated table pointer. Calls through the interface use that table
for runtime dispatch; calls through a concrete class retain static dispatch.
The representation is internal and does not expose pointers in Princi syntax.

### Traits and interfaces (v0.2 development)

An `interface` is a pure behavioral contract: every declared method is
required, and interfaces cannot contain method bodies. A `trait` is also a
behavioral contract, but a trait method may have a default body. Traits cannot
contain fields or constructors. Both use `implements` on a class; traits may
also be implemented by structs.

```princi
trait Named {
    fn name() -> String

    fn describe() {
        println(self.name())
    }
}

class User implements Named {
    value: String

    init(value: String) {
        self.value = value
    }

    fn name() -> String {
        return self.value
    }
}
```

An explicit method overrides a default. A type must implement every abstract
trait method with a matching signature. If multiple implemented traits provide
the same default method, the type must declare that method explicitly or the
compiler reports an ambiguity. There are no automatic conflict-precedence
rules.

Trait-typed values reuse the interface runtime pair and dispatch methods through
a generated table. A class trait view keeps the class reference. Converting a
struct to a trait view boxes a copy in process-lifetime managed storage, so
later changes to the original struct are not visible through that view. Struct
methods are currently supported only on structs that implement a trait, and
have read-only `self`. Traits are non-generic.

### Bounded generics (v0.2 development)

Generic functions, classes, and structs can require interface and/or trait
contracts. Write one bound after `:` and additional bounds separated by `+`:

```princi
trait Named { fn name() -> String }
interface Printable { fn print() }

fn show<T: Named + Printable>(value: T) {
    println(value.name())
    value.print()
}

class User implements Named, Printable {
    fn name() -> String { return "Ada" }
    fn print() { println("user") }
}

class Repository<T: Named> {
    value: T
}

struct Holder<T: Named> {
    value: T
}

fn main() {
    show(User())
}
```

At a generic function call or generic class/struct instantiation, every bound is
checked against the concrete type. A bounded type parameter can call methods
declared by its bounds. If multiple bounds declare the same method, their
signatures must match. Generic functions remain monomorphized, so a bound call
lowers against the concrete specialization; no erased generic dispatch is
introduced. Interface bounds accept conforming classes, while trait bounds
accept conforming classes and structs. Failed checks produce a short diagnostic
such as `type Int does not satisfy required trait 'Named'`. Generic traits,
associated types, and conditional or negative bounds are unsupported.

### Cross-feature compatibility (v0.2 development)

Generic arguments compose recursively across the currently supported types.
Examples include `List<Result<Int, String>>`,
`Result<List<Int>, String>`, `Box<Result<User, IOError>>`, and
`Pair<Result<User, IOError>, Point>`. Generic classes and structs receive one
deterministic native specialization for each concrete type argument set used;
repeated uses reuse that specialization. Generic functions follow the same
monomorphized model.

Enums, including `Maybe<T>` and the prelude `Result<T, E>`, use the ordinary
tagged value representation regardless of their generic payloads. Payloads can
contain primitives, structs, class references, Strings, Lists, other enums,
and interface or trait values. `List<T>` stores a value copy of each element;
inline structs/enums are copied, while pointers embedded in them and the
data/vtable pair of a contract value are copied as part of the value. For
example, `List<Drawable>` and `List<Result<Drawable, String>>` preserve the
concrete dispatch table through indexing, function calls, and pattern matching.

Interface and trait values retain their dispatch table across locals,
parameters, return values, fields, lists, and enum payloads where the semantic
type is supported. A struct converted to a trait value is boxed as a copy, as
described above. Generic bounds are checked at specialization time; bounded
generic calls are monomorphized and do not erase the concrete type.

The current runtime uses process-lifetime managed allocation. It does not
collect individual unreachable values or traverse references in Lists,
structs, or enum payloads. This is safe for the supported v0.2 value model
because managed allocations are not reclaimed while Princi code is running;
all registered allocations are released together when the program exits.
Consequently, nested managed references and interface/trait dispatch data stay
valid through copies and list growth. Long-running programs can retain
unreachable allocations until shutdown; a tracing collector is still future
work.

Postfix `?` works with generic `Result<T, E>` values and nested payload types.
It returns the original error payload without conversion, so the enclosing
function's error type must match exactly. It does not introduce a separate
representation for `Result` or bypass ordinary enum matching and typing.

```princi
class Box<T> {
    value: T

    init(value: T) {
        self.value = value
    }

    fn get() -> T {
        return self.value
    }
}

struct Pair<A, B> {
    first: A
    second: B
}

fn main() {
    let number = Box(42) // inferred as Box<Int>
    let pair = Pair<Int, String>(1, "one")
    println(number.get())
    println(pair.second)
}
```

The canonical boundary, compatibility expectations, and explicit exclusions
are documented in [the PrinciPL v0.2 scope](docs/v0.2-scope.md). Features listed
as outside v0.2 are not current functionality or release commitments.

## Windows setup

Princi v0.1 targets Windows x86-64 and requires three tools:

1. Install the stable Rust toolchain and Cargo with
   [rustup](https://rust-lang.org/tools/install/).
2. Install LLVM/Clang 15 or newer from the [official LLVM releases](https://releases.llvm.org/).
   Add its `bin` directory to `PATH`. Check `clang --version` and verify that
   `clang --print-targets` lists X86.
3. Install [MSYS2](https://www.msys2.org/). Open its UCRT64 terminal, update
   packages with `pacman -Syu`, reopen the terminal, then install the x86-64
   [MinGW-w64 GCC package](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-gcc?repo=ucrt64)
   with:

   ```sh
   pacman -S mingw-w64-ucrt-x86_64-gcc
   ```

   Add the MSYS2 `ucrt64\bin` directory (normally `C:\msys64\ucrt64\bin`) to
   Windows `PATH`. Check `gcc -dumpmachine`; it must print
   `x86_64-w64-mingw32`.

The compiler reads `clang` and `gcc` from `PATH` by default. Set
`PRINCI_CLANG` or `PRINCI_CC` to select a different executable. Missing or
incompatible tools produce compiler diagnostics. Windows Rust installation
may also prompt you to install the Visual Studio C++ Build Tools needed by the
Rust host toolchain.

## Build and install the compiler

From a clean clone, build the debug CLI and release CLI with Cargo:

```powershell
git clone https://github.com/0xPolybit/principl.git
cd principl
cargo build
cargo build --release
```

The binaries are `target\debug\princi.exe` and
`target\release\princi.exe`. To install the release CLI into Cargo's binary
directory, run:

```powershell
cargo install --path .
```

If `princi` is not found, add `%USERPROFILE%\.cargo\bin` to `PATH` and open a
new terminal. The v0.1 CLI has one command: `princi build <source-file>`.

### Install a Windows release

Download the x64 installer from the
[latest GitHub Release](https://github.com/0xPolybit/principl/releases/latest)
and run it. The installer places `princi.exe` in your per-user programs folder
and offers to add its `bin` directory to your user `PATH`. Open a new terminal
after installation, then use `princi build ...` as shown below. You do not need
Rust or Cargo to run the prebuilt compiler.

The release installer does not bundle LLVM or MinGW-w64. The compiler still
requires the LLVM/Clang and x86-64 MinGW-w64 tools described in [Windows
setup](#windows-setup) on the machine that builds Princi programs. The
installer can also add a **Build with Princi** Explorer action for `.prnc` and
`.princi` files; this leaves your default source editor unchanged. A portable
ZIP is available on the same release page for users who prefer to place the
compiler manually.

## Compile a Princi program

Save this source as `hello.prnc` or `hello.princi`:

```princi
fn main() {
    print("Hello from Princi!")
}
```

From the directory containing the source, run either command:

```powershell
princi build hello.prnc
princi build hello.princi
```

Both commands produce `hello.exe` beside the input. Launch it with:

```powershell
.\hello.exe
```

It prints `Hello from Princi!` without a trailing newline. Select another
output path with `-o`:

```powershell
princi build hello.prnc -o app.exe
```

This produces `app.exe`; relative `-o` paths are resolved from the current
working directory. The compiler automatically runs the lexer, parser, semantic
analysis, LLVM IR verification, Windows object generation, and MinGW linker.
Failed builds exit non-zero and do not publish a partial executable.

## Run the tests

Run all unit, CLI, and fixture-driven checks with:

```powershell
cargo test
```

On Windows with LLVM/Clang and x86-64 MinGW-w64 GCC available, native tests
build generated `.exe` files and check their output and exit status. If the
native toolchain is missing, those tests print a skip note when output capture
is disabled:

```powershell
cargo test -- --nocapture
```

Set `PRINCI_REQUIRE_NATIVE_TESTS=1` to make missing native tools fail the test
run instead of skipping the executable checks:

```powershell
$env:PRINCI_REQUIRE_NATIVE_TESTS = "1"
cargo test
```

Fixtures under `tests/fixtures` include a full feature program and identical
`.prnc`/`.princi` hello programs. Negative fixtures check diagnostic codes and
that failed builds do not produce executables.

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
unsupported imports. `E0210` reports a local read before definite
initialization. In the v0.2 work, `E0211` identifies invalid interface
conformance and `E0212` identifies invalid trait implementations, missing
methods, and default-method ambiguities. `E0213` reports invalid generic
bounds or a concrete type that does not satisfy a required interface or trait.

Toolchain errors are separate from source errors: `E0401` means LLVM/Clang is
missing, `E0403` means the Windows MinGW linker is missing or targets the wrong
platform, and `E0404` means linking failed. Install LLVM/Clang with LLVM IR and
X86 support and x86-64 MinGW-w64 GCC, or point `PRINCI_CLANG` and `PRINCI_CC` at
those tools. A generated-IR failure is reported as an internal compiler error
(`E9002`); set `PRINCI_KEEP_INTERMEDIATES=1` to preserve its `.ll` input for
investigation. Unexpected compiler failures use `E9001` and exit non-zero
without exposing a Rust panic or backtrace.

## Website development

The separate Next.js website lives in [`frontend/`](frontend/README.md). From
the repository root, start it locally with:

```powershell
cd frontend
npm install
npm run dev
```

See [`frontend/README.md`](frontend/README.md) for architecture, content
maintenance, lint, production build, and deployment instructions.
