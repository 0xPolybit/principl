Princi v0.1 manages ordinary heap allocations automatically. String values, class instances, and `List<T>` values are managed references in the compiler's type system. Structs are inline values, although their fields may contain managed references. Source code has no raw-pointer type and no manual allocate/free operation.

## Process-lifetime heap

The private runtime ABI registers dynamic string concatenation buffers, class instances, list headers, and list backing buffers in a process-lifetime heap registry. The generated entry wrapper releases registered storage after Princi `main` returns. Runtime failure paths also release it. String literals are immutable module constants and need no heap allocation.

## Trade-offs

The v0.1 runtime does not reclaim individual unreachable values while a program runs. Temporaries, cycles, and old list buffers remain allocated until process exit, so peak memory can grow in a long-running allocation-heavy program. This simple policy keeps aliases valid without source-level lifetime rules.

A future tracing collector could replace the private allocation boundary without changing source syntax. Tracing collection, generational/concurrent GC, ownership, borrowing, pinning, raw pointers, unsafe blocks, and user-controlled arenas are not v0.1 features.
