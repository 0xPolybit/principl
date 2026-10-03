Princi v0.1 is deliberately narrow. The repository scope document is the authority for the current language boundary; possible future work does not describe available compiler behavior.

## Deferred areas

- A richer managed-memory model, potentially tracing collection.
- Optional ownership/borrowing concepts and lower-level memory access.
- More target platforms.
- More expressive generic types and a package ecosystem.
- Async execution and trait-like abstractions.

These are directions rather than promises or scheduled releases. Design work must preserve the current explicit boundary until implementation and tests support a feature.

## Current boundary

The v0.1 compiler targets Windows x86-64, uses process-lifetime managed allocations released at program exit, supports only the built-in `io` and `math` module names, and exposes only the documented primitive C FFI signatures. See [Limitations](/docs/reference/limitations) and the root [v0.1 scope document](https://github.com/0xPolybit/principl/blob/main/docs/v0.1-scope.md).
