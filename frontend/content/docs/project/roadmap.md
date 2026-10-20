PrinciPL v0.2.0 is the current release. The [canonical v0.2 scope](https://github.com/0xPolybit/principl/blob/main/docs/v0.2-scope.md) describes implemented behavior. The planned v0.3 areas below are not available in the current compiler and do not promise a release date.

## Planned for v0.3

- Async functions, `Task<T>`, `await`, structured task scopes, cancellation, and timers.
- OS threads with `Thread<T>`, join, channels, mutexes, limited atomics, and conservative type-based transfer rules.
- Project manifests (`princi.toml`), lockfiles (`princi.lock`), user modules, visibility, local/Git dependencies, and deterministic dependency caching.
- Project-aware CLI commands while preserving `princi build file.prnc`, `princi build file.princi`, and `-o`.

The compiler target remains Windows x86-64. Async tasks may share ordinary managed values inside the cooperative async runtime; OS threads use stricter rules and cannot capture unrestricted mutable managed references. See the [canonical v0.3 scope](https://github.com/0xPolybit/principl/blob/main/docs/v0.3-scope.md) for semantics, boundaries, and compatibility requirements.

## Current v0.2 boundary

The current compiler uses process-lifetime managed allocations released at program exit, supports only the built-in `io` and `math` module names, and exposes only the documented primitive C FFI signatures. Generics, bounds, interfaces, traits, enums, matching, `Result`, and `?` are implemented. Async, threads, project manifests, and user-defined modules are not implemented. See [Limitations](/docs/reference/limitations) and the root [v0.2 scope document](https://github.com/0xPolybit/principl/blob/main/docs/v0.2-scope.md).

## Memory directions beyond v0.3

v0.2 and planned v0.3 retain process-lifetime allocation. Possible v0.4/v0.5 research includes deterministic lifetimes, scoped arenas, limited borrowing, tracing/concurrent collection, and carefully bounded lower-level memory APIs. These are exploratory directions, not promised features or a fixed release schedule. Ownership, borrowing, arenas, raw pointers, unsafe blocks, manual allocation/free, destructors, and tracing collection are outside v0.3.
