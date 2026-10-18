PrinciPL 0.2 is a Windows-native compiler with statically checked procedural and object-oriented code. It supports generic functions, classes, structs, and enums; monomorphization; trait and interface bounds; dynamic interface/trait values; algebraic enums and exhaustive matching; `Result<T, E>` and `?`; the v0.1 procedural core; `List<T>`; built-in modules; and a restricted C ABI.

## Current v0.2 boundaries

- Generic inference uses call or constructor arguments only; it does not infer
  from an expected result type. Generic enum variant construction requires
  explicit type arguments. Methods cannot introduce their own type parameters,
  and traits/interfaces cannot be generic.
- Generic code is monomorphized at build time. A build accepts at most 512
  distinct generic function specializations and 512 generic type layouts.
- Interface values can wrap class instances only. Structs can implement traits;
  converting a struct to a trait value boxes a copy in managed storage.
- Enum layouts are compiler-managed, not C-compatible or user-customizable.
  Match guards, list/array patterns, struct destructuring, and range patterns
  are unsupported. Primitive matches need a catch-all except for `Bool`, whose
  `true` and `false` cases are finite.
- `?` requires a `Result<T, E>` operand and an enclosing function returning a
  `Result` with exactly the same error type `E`. The program entry point may
  return only `Void` or `Int`.

For the syntax and examples behind these limits, see [Generics](/docs/language/generics),
[Interfaces](/docs/language/interfaces), [Traits](/docs/language/traits),
[Enums](/docs/language/enums), [Pattern matching](/docs/language/match), and
[Result propagation](/docs/language/result).

## Not implemented

- Non-Windows targets, including Linux, macOS, and WebAssembly.
- Package management, remote packages, user-defined multi-file modules, and filesystem import resolution.
- REPL, formatter, documentation generator, debugger, IDE integration, and reflection.
- Raw pointers, unsafe blocks, ownership, borrowing, arenas, pinning, or manual memory operations.
- Async/await, class inheritance, virtual class methods, abstract classes, and overloads.
- Additional collection types, iterators, comprehensions, and higher-order collection functions.
- General C pointers, callbacks, variadic FFI, C aggregate layout, and Python interoperability.

Parser support and native support are separate boundaries: syntax outside the lowered subset can receive an explicit backend diagnostic. Consult the [canonical v0.2 scope](https://github.com/0xPolybit/principl/blob/main/docs/v0.2-scope.md) for exact rules.
