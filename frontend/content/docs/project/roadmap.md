PrinciPL v0.2 is the current release. The repository's canonical v0.2 scope document is the authority for current behavior; possible future work does not describe available compiler behavior or a release commitment.

## Deferred areas

- A richer managed-memory model, potentially tracing collection.
- Optional ownership/borrowing concepts and lower-level memory access.
- More target platforms.
- A package ecosystem and broader module resolution.
- Async execution and threading language support.

These are directions rather than promises or scheduled releases. Design work must preserve the current explicit boundary until implementation and tests support a feature.

## Current boundary

The v0.2 compiler targets Windows x86-64, uses process-lifetime managed allocations released at program exit, supports only the built-in `io` and `math` module names, and exposes only the documented primitive C FFI signatures. Generics, bounds, interfaces, traits, enums, matching, `Result`, and `?` are implemented in v0.2. See [Limitations](/docs/reference/limitations) and the root [v0.2 scope document](https://github.com/0xPolybit/principl/blob/main/docs/v0.2-scope.md).
