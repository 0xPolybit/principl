<p align="center">
  <img src="frontend/app/icon.svg" alt="PrinciPL logo" width="96" height="96" />
</p>

<h1 align="center">PrinciPL</h1>

<p align="center">
  <strong>Structure when you need it. Simplicity when you don't.</strong><br />
  A statically typed programming language compiled to native Windows x86-64 executables.
</p>

<p align="center">
  <a href="https://github.com/0xPolybit/principl/actions/workflows/ci.yml"><img alt="Compiler CI" src="https://github.com/0xPolybit/principl/actions/workflows/ci.yml/badge.svg?branch=main" /></a>
  <a href="docs/v0.2-scope.md"><img alt="Compiler version 0.2.0" src="https://img.shields.io/badge/compiler-v0.2.0-557a63" /></a>
  <a href="docs/v0.2-scope.md"><img alt="Target: Windows x86-64" src="https://img.shields.io/badge/target-Windows%20x86--64-557a63" /></a>
  <a href="docs/v0.2-scope.md"><img alt="Source extensions: .prnc and .princi" src="https://img.shields.io/badge/source-.prnc%20%2F%20.princi-557a63" /></a>
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-blue.svg" /></a>
</p>

<p align="center">
  <a href="https://github.com/0xPolybit/principl/releases">Published Windows releases</a> ·
  <a href="docs/v0.2-scope.md">Language scope</a> ·
  <a href="frontend/">Website source</a>
</p>

PrinciPL 0.2.0 is implemented in Rust and uses LLVM IR, Clang, and MinGW-w64
to build native Windows x86-64 programs. It combines concise procedural syntax
with generic types, classes, structs, interfaces, traits, algebraic enums,
pattern matching, and `Result`-based error propagation.

> The compiler and language currently target Windows x86-64 only. The
> `.prnc` and `.princi` extensions are interchangeable aliases for the same
> source language.

## At a glance

- **Static types:** compile-time checks for values, calls, generic bounds, and
  enum matches.
- **Native builds:** compile a source file to a Windows `.exe` with one command.
- **Structured code:** functions, classes, value-oriented structs, interfaces,
  traits, and generic types.
- **Explicit outcomes:** algebraic enums, exhaustive `match`, `Result<T, E>`,
  and `?` error propagation.
- **Automatic storage lifetime:** managed allocations are retained until the
  process exits; v0.2 does not include a tracing garbage collector.

## Quick start

Install a published Windows build from the [GitHub Releases page](https://github.com/0xPolybit/principl/releases),
or install the compiler from this source checkout after installing stable Rust:

> Release downloads correspond to the tag shown on the selected release. This
> README documents the v0.2.0 source in this repository; check the release tag
> to confirm the version of a packaged compiler.

```powershell
git clone https://github.com/0xPolybit/principl.git
cd principl
cargo install --path .
```

To compile Princi programs, also install LLVM/Clang and x86-64 MinGW-w64 as
described in [Windows setup](#windows-setup). Save this program as `hello.prnc`
or `hello.princi`:

```princi
fn main() {
    println("Hello from PrinciPL!")
}
```

Build and launch it in PowerShell:

```powershell
princi build hello.prnc
.\hello.exe
```

Use `princi build hello.princi` for the alias extension, or pass `-o app.exe`
to choose the output path. The rest of this README documents the v0.2 feature
boundary, toolchain, architecture, and tests.

## Contents

- [v0.2 feature highlights](#v02-feature-highlights)
- [Windows setup](#windows-setup)
- [Install and build the compiler](#install-and-build-the-compiler)
- [Examples](#examples)
- [Compiler architecture](#compiler-architecture)
- [Tests and validation](#tests-and-validation)
- [Current language boundaries](#current-language-boundaries)
- [Roadmap beyond v0.2](#roadmap-beyond-v02)

## v0.2 compiler contract

The only user-facing compiler workflow is:

```powershell
princi build <source.prnc|source.princi> [-o output.exe]
```

`.prnc` and `.princi` are exact aliases for the same source language. Either
input below produces `hello.exe` beside its source:

```powershell
princi build hello.prnc
princi build hello.princi
```

Choose another executable path with `-o`:

```powershell
princi build hello.prnc -o app.exe
```

The compiler targets Windows x86-64 only. A failed build returns a nonzero
status, reports a diagnostic, and does not publish a partial executable.

## v0.2 feature highlights

- Statically checked `Int`, `Float`, `Bool`, `String`, `Void`, class, struct,
  enum, interface, trait-object, and generic instance types.
- Generic functions, classes, structs, and enums with nested type arguments,
  local generic inference where unambiguous, interface/trait bounds, and
  compile-time monomorphization.
- Classes with managed reference semantics; structs with value-copy semantics;
  interfaces with dynamic dispatch; traits with required and default methods.
- Algebraic enums, payload destructuring, match expressions, wildcard arms,
  exhaustiveness checks, and the standard `Result<T, E>` enum.
- Postfix `?` propagation between functions whose `Result` error types match.
- v0.1 procedural features, `List<T>`, built-in imports, the restricted C ABI,
  diagnostics, and Windows native build pipeline remain supported.

### Generics and specialization

Generic function arguments can be explicit or inferred from call arguments when
every parameter has one unambiguous concrete type. The compiler emits only
used function specializations, reuses equivalent instances, and gives them
stable internal names. Same-type recursive calls work; infinitely expanding
specializations receive a diagnostic. Generic class and struct layouts are
specialized by concrete type arguments. Generic constructors can infer their
arguments from constructor parameters when possible.

```princi
fn identity<T>(value: T) -> T {
    return value
}

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
    let number = identity(42)
    let box = Box<Int>(number)
    let pair = Pair<Int, String>(box.get(), "forty-two")
    println(pair.second)
}
```

### Interfaces, traits, and bounds

An interface is a method-signature contract. Classes can implement multiple
interfaces; values viewed through an interface use dynamic dispatch. A trait
is a method contract that may include defaults. Classes and structs can
implement traits; explicit methods override defaults. Conflicting defaults
require an explicit implementation. Interfaces and traits have no fields,
constructors, or generic methods.

Generic functions, classes, and structs may require interface or trait bounds.
The compiler checks every bound before specializing code and permits calls on
the generic value only for methods promised by those bounds.

```princi
trait Printable {
    fn display() -> String
    fn show() {
        println(self.display())
    }
}

fn showValue<T: Printable>(value: T) {
    value.show()
}
```

Interface values currently represent class instances. Structs can implement
traits; converting a struct to a trait value boxes a copy in managed storage.
There is no class inheritance or virtual class dispatch.

### Enums, `match`, and `Result`

Enums have named variants with optional positional payloads. Generic arguments
are written on the enum type when constructing a variant:

```princi
enum State {
    Ready
    Failed(String)
}

fn main() {
    let state = State.Failed("offline")
    match state {
        Ready => println("ready")
        Failed(reason) => println(reason)
    }
}
```

`match` supports enum variants, payload bindings, wildcard/binding patterns,
and primitive literal patterns. Enum matches must cover every variant or use a
catch-all. Match expressions require compatible arm types. Array/list and
struct destructuring, guards, and range patterns are not supported.

`Result<T, E>` is supplied by the embedded source prelude as an ordinary
generic enum. Constructors use qualified type syntax such as
`Result<Int, String>.Ok(42)`; patterns use contextual `Ok(value)` and
`Err(error)` names. The postfix `?` operator extracts the success value or
returns the error from a compatible `Result`-returning function. Error types
must match exactly; implicit error conversions are not performed.

```princi
fn calculate(value: Int) -> Result<Int, String> {
    if value < 0 {
        return Result<Int, String>.Err("value must be positive")
    }
    return Result<Int, String>.Ok(value * 2)
}

fn run() -> Result<Int, String> {
    let doubled = calculate(21)?
    return Result<Int, String>.Ok(doubled)
}

fn main() {
    match run() {
        Ok(value) => println(value)
        Err(error) => println(error)
    }
}
```

The program prints `42`.

## v0.1 compatibility and migration

Princi 0.2 preserves v0.1 syntax and behavior, including both source
extensions, the `princi build` command, primitive types, variables, functions,
control flow, classes, structs, lists, built-in imports, and the restricted C
FFI. Existing `.prnc` and `.princi` projects do not need a source migration.

New words such as `interface`, `trait`, `implements`, and `enum` are parsed in
their declaration contexts so they do not silently change the established
v0.1 identifier rules. The compiler remains Windows x86-64 only; v0.2 does not
add a package manager or new top-level CLI commands.

## Current language boundaries

- Generic methods are not supported; methods can use type parameters from their
  enclosing generic class.
- Generic inference uses call/constructor arguments, not an expected result
  type. Ambiguous or unused parameters require explicit type arguments.
- Interfaces are implemented by classes only. Structs implement traits, not
  interfaces. Trait and interface definitions are not generic.
- Trait default conflicts must be resolved by defining the method on the
  concrete class or struct. There is no automatic precedence.
- Match does not support guards, list/array patterns, struct destructuring, or
  range patterns. Primitive literal matches need a catch-all unless the type is
  finite (`Bool`).
- The C FFI accepts only `Int32`, `Int64`, and `Float64` parameters, with
  `Int32`, `Int64`, `Float64`, or `Void` returns. Managed values and pointers
  cannot cross this boundary.
- Imports resolve only the built-in `io` and `math` names. `math` currently
  exports no functions; arbitrary user modules and packages are not resolved.
- Managed heap storage is retained until program termination and then released
  together. There is no tracing collector or source-level ownership system.
- No inheritance, raw pointers, unsafe blocks, manual memory operations,
  ownership/borrowing, arenas, async/await, threading syntax, package manager,
  Linux/macOS targets, WebAssembly, macros, reflection, operator overloading,
  or Python interoperability.

See the [canonical v0.2 scope](docs/v0.2-scope.md) for the full supported
boundary and architecture.

## Compiler architecture

The build pipeline is:

```text
.prnc / .princi
  → source loading
  → lexer
  → parser
  → AST
  → built-in module resolution
  → semantic analysis and type checking
  → typed representation
  → generic specialization
  → LLVM IR verification
  → Windows x86-64 COFF object
  → MinGW-w64 linker and runtime
  → .exe
```

LLVM IR and object files live in a temporary build directory and are removed
after a successful build. Set `PRINCI_KEEP_INTERMEDIATES=1` to retain them for
development. Compiler modules are organized around `cli`, `diagnostics`,
`source`, `lexer`, `parser`, `ast`, `modules`, `ffi`, `semantic`, `types`,
`codegen`, and `runtime`.

## Managed memory

Princi class instances, strings that need dynamic storage, lists, and other
runtime-managed allocations use a private runtime allocation boundary. The
current implementation keeps registered allocations alive for the whole
process and releases them when Princi `main` returns. This is automatic
process-lifetime management, not a tracing or generational garbage collector.
It preserves references and cycles, but temporary objects and grown list
buffers remain allocated until exit. No manual allocation or free operation is
available in Princi source.

## Windows setup

Building the compiler itself requires stable Rust and Cargo. Building a Princi
program also requires LLVM/Clang with LLVM IR input and X86 support, plus
x86-64 MinGW-w64 GCC.

1. Install stable Rust with [rustup](https://rust-lang.org/tools/install/).
   Windows may also prompt for Visual Studio C++ Build Tools for the Rust host.
2. Install LLVM/Clang 15 or newer and add its `bin` directory to `PATH`.
   Verify `clang --version` and confirm `clang --print-targets` includes X86.
3. Install [MSYS2](https://www.msys2.org/), use its UCRT64 terminal, and run:

   ```sh
   pacman -Syu
   pacman -S mingw-w64-ucrt-x86_64-gcc
   ```

   Add `C:\msys64\ucrt64\bin` to Windows `PATH`. `gcc -dumpmachine` must
   report `x86_64-w64-mingw32`.

The compiler finds `clang` and `gcc` on `PATH` by default. Set
`PRINCI_CLANG` or `PRINCI_CC` to select another executable. Missing or
incompatible tools produce diagnostics. The prebuilt compiler installer does
not bundle LLVM or MinGW-w64.

## Install and build the compiler

### Install a published Windows build

Download `Princi-Setup-<version>-x64.exe` or the portable ZIP from the
[GitHub Releases page](https://github.com/0xPolybit/principl/releases). The
installer can add `princi` to the current user's `PATH` and add a **Build with
Princi** Explorer action for both source extensions. Open a new terminal after
installing. Release artifacts use the version shown on that release page.
Neither package includes LLVM/Clang or MinGW-w64; install those separately as
described in [Windows setup](#windows-setup).

### Build from source

```powershell
git clone https://github.com/0xPolybit/principl.git
cd principl
cargo build
cargo build --release
```

The binaries are `target\debug\princi.exe` and
`target\release\princi.exe`. Install the current checkout into Cargo's user
binary directory with:

```powershell
cargo install --path .
```

If `princi` is not found, add `%USERPROFILE%\.cargo\bin` to `PATH` and open a
new terminal.

## Examples

The repository's [`examples/`](examples/) folder contains source programs. A
minimal program is:

```princi
fn main() {
    println("Hello from PrinciPL!")
}
```

Save it as `hello.prnc` or `hello.princi`, then build and run it in PowerShell:

```powershell
princi build hello.prnc
.\hello.exe
```

Both extensions produce the same executable name and behavior. Use
`princi build hello.princi -o app.exe` to select an explicit output path.

## Tests and validation

Run formatting, lint, tests, and both compiler profiles from the repository
root:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
cargo build --release
```

On Windows, the native integration suite requires LLVM/Clang with X86 support
and x86-64 MinGW-w64 GCC. Set `PRINCI_REQUIRE_NATIVE_TESTS=1` to make missing
native tools fail the test run rather than skip it:

```powershell
$env:PRINCI_REQUIRE_NATIVE_TESTS = "1"
cargo test
```

The suite covers unchanged v0.1 fixtures, paired `.prnc`/`.princi` v0.2
conformance builds, executable output/exit status, invalid-program diagnostics,
LLVM lowering, generics, bounds, interfaces, traits, enums, matching, and
`Result` propagation. See the [v0.2 testing matrix](docs/v0.2-testing-matrix.md).

## Diagnostics

Errors include the source file, line, column, diagnostic code, source line,
and caret span. Common codes include `E0002` unsupported extension, `E0003`
missing source, `E0100` lexical error, `E0101` syntax error, `E0201` type
mismatch, `E0211` interface conformance, `E0212` trait implementation,
`E0213` generic bound, `E0302` generic arity, `E0303` generic inference,
`E0304` enum pattern/variant, `E0305` match analysis, `E0306` `?`
propagation, and `E0307` specialization failure. `E0401`/`E0403` report
missing LLVM/Clang or linker tools; `E0404` reports link failure. Unexpected
internal failures return `E9001` instead of a Rust panic trace.

## Roadmap beyond v0.2

PrinciPL v0.2.0 remains the current release. The planned v0.3 scope focuses on
async programming and structured concurrency, conservative OS-thread
synchronization, and project/module/package tooling. It preserves the current
Windows x86-64 target and the standalone `.prnc` / `.princi` build workflow.
These features are not available in v0.2. See the [canonical v0.3 scope](docs/v0.3-scope.md) for boundaries, compatibility expectations, and the tentative v0.4/v0.5 memory-system research areas.

### v0.3 async syntax foundation

The frontend now recognizes the planned async/concurrency forms:

```princi
async fn fetch() -> String {
    return "ready"
}

async fn main() {
    let value = await fetch()

    scope {
        let first = spawn fetch()
        println(await first)
    }
}
```

`async fn` is currently recognized for top-level functions. `await` is a prefix
expression at unary precedence: it applies to the next call/member/index
expression, before surrounding binary operators. It is accepted only inside
an `async fn`. `scope { ... }` is parsed as a task-scope statement. The
reserved OS-thread form is `thread spawn worker(args)`.

This is parser/AST groundwork only. Async functions, `await`, `spawn`, task
scopes, and thread spawning do not have runtime or LLVM support yet; attempting
to build programs that use them produces an unsupported-feature diagnostic.
The current v0.2 release remains synchronous. `.prnc` and `.princi` accept the
same syntax and produce the same AST.

## Website development

The independent Next.js website lives in `/frontend`:

```powershell
cd frontend
npm install
npm run dev
```

See [`frontend/README.md`](frontend/README.md) for website development and
deployment instructions.
