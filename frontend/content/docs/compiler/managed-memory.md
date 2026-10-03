Princi v0.1 automatically manages ordinary runtime heap allocations, but it does not implement a tracing garbage collector. The current policy is process-lifetime retention: allocated blocks remain valid through the program's execution and are released together when Princi `main` returns.

## What is managed today

In the compiler type system, `String`, class instances, and `List<T>` are managed references. The Windows runtime registers dynamic string-concatenation buffers, class-instance storage, list headers, list backing buffers, and replacement buffers allocated while lists grow. String literals are immutable constants in the generated module and do not need heap allocation. Structs are inline values, although their fields may hold a managed reference.

The private allocator places a small link header before each managed block and adds it to a runtime registry. At shutdown, the runtime walks that registry and releases every block. Runtime error paths also perform shutdown before process exit.

~~~text
allocation → register block → keep valid while program runs
           → release registered blocks together at shutdown
~~~

## What this means for programs

Princi source has no `free`, manual allocation, or raw-pointer operation. Assigning a class or list copies its reference, so aliases remain valid. Strings created by concatenation also stay valid until shutdown.

The runtime does not reclaim an unreachable value during execution. Temporary values, cycles, and old list buffers remain registered until exit. This avoids use-after-free from premature reclamation, but allocation-heavy long-running programs can retain increasing amounts of memory.

## Current v0.1 limitations

There is no tracing or generational collection, concurrent collection, cycle detection, finalization, pinning, borrowing, ownership syntax, or user-controlled arena. The process-lifetime registry is the actual v0.1 implementation; it is not a proxy for a collector that already exists.

## Where PrinciPL is going

PrinciPL's longer-term design concept is to let developers use automatic management by default and introduce more direct lifetime control progressively when a program needs it. The diagram distinguishes the current first step from future concepts:

<!-- princi-diagram:memory-roadmap -->

“Deterministic / owned,” “Arena,” and “Raw / manual” are roadmap ideas, not current syntax, APIs, guarantees, or a release schedule. No ownership, borrowing, arena, raw pointer, or manual-free feature is part of v0.1.

## Related topics

- [Runtime](/docs/compiler/runtime)
- [Type system: managed references](/docs/compiler/type-system)
- [Classes](/docs/language/classes)
- [Lists](/docs/language/lists)
- [Explicit v0.1 limitations](/docs/reference/limitations)
