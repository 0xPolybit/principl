PrinciPL 0.2 is a Windows-native compiler with statically checked procedural and object-oriented code. It supports generic functions, classes, structs, and enums; monomorphization; trait and interface bounds; dynamic interface/trait values; algebraic enums and exhaustive matching; `Result<T, E>` and `?`; the v0.1 procedural core; `List<T>`; built-in modules; and a restricted C ABI.

## Not implemented

- Non-Windows targets, including Linux, macOS, and WebAssembly.
- Package management, remote packages, user-defined multi-file modules, and filesystem import resolution.
- REPL, formatter, documentation generator, debugger, IDE integration, and reflection.
- Raw pointers, unsafe blocks, ownership, borrowing, arenas, pinning, or manual memory operations.
- Async/await, class inheritance, virtual class methods, abstract classes, and overloads.
- Additional collection types, iterators, comprehensions, and higher-order collection functions.
- General C pointers, callbacks, variadic FFI, C aggregate layout, and Python interoperability.

Parser support and native support are separate boundaries: syntax outside the lowered subset can receive an explicit backend diagnostic. Consult the [canonical v0.2 scope](https://github.com/0xPolybit/principl/blob/main/docs/v0.2-scope.md) for exact rules.
