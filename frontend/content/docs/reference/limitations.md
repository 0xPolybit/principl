v0.1 is a focused Windows-native compiler. Its implemented subset includes typed functions and recursion, local `let`/`var` bindings, checked expressions, control flow, strings and primitive printing, classes with static dispatch, field-only value structs, `List<T>`, built-in module names, and a restricted C ABI.

## Not implemented

- Non-Windows targets, including Linux, macOS, and WebAssembly.
- Package management, remote packages, user-defined multi-file modules, and filesystem import resolution.
- REPL, formatter, documentation generator, debugger, IDE integration, and reflection.
- Raw pointers, unsafe blocks, ownership, borrowing, arenas, pinning, or manual memory operations.
- Async/await, traits, interfaces, inheritance, virtual/abstract methods, and overloads.
- General-purpose generics, additional collection types, iterators, comprehensions, and higher-order collection functions.
- General C pointers, callbacks, variadic FFI, C aggregate layout, and Python interoperability.

Parser support and native support are separate boundaries: syntax outside the lowered subset can receive an explicit backend diagnostic. Consult the [scope document](https://github.com/0xPolybit/principl/blob/main/docs/v0.1-scope.md) for exact rules.
