Semantic analysis runs after parsing and built-in module resolution. It checks that the AST is meaningful under the v0.1 language rules and produces the typed representation used by code generation.

## Declaration and scope passes

The analyzer first registers global function and type names, then resolves function signatures and per-type members before checking bodies. This lets declarations refer to nominal types registered elsewhere in the same source file. It tracks functions and external functions separately from class and struct symbols.

Local scopes are lexical. Parameters and locals are recorded with their type, mutability, and initialization state. Nested blocks and loop bodies create scopes; shadowing an outer local is allowed, but duplicate names in one scope are rejected. The analyzer also checks class fields and methods, struct fields, initializers, and constructor signatures.

## Expression and statement checks

The analyzer resolves identifiers and members, checks call arity and argument types, checks assignments and operators, and requires `Bool` conditions. It verifies return values and that non-`Void` functions return on every path. A local must be definitely initialized before it is read; assignments in both arms of `if/else` can establish initialization, but loop-body assignments cannot because the loop may not run.

~~~princi
fn main() {
    var count: Int
    count = 3
    println(count)
}
~~~

## Output and stage boundary

On success, `semantic::analyze_with_modules` returns `TypedProgram` with the AST, global/type symbols, resolved modules, and type maps keyed by source spans. On failure, it returns source diagnostics. `compiler::pipeline` stops before LLVM generation whenever semantic diagnostics exist.

~~~princi
fn main() {
    let age: Int = "twenty"
}
~~~

This assignment is rejected as a type mismatch; the backend never sees it as valid typed input.

## Current v0.1 limitations

Type inference is local to initialized variable declarations and non-empty list literals. There is no advanced generic inference, implicit numeric conversion, overload resolution, inheritance lookup, or cross-file user module resolution. The only source-level generic collection is `List<T>`.

## Related topics

- [AST](/docs/compiler/ast)
- [Type system](/docs/compiler/type-system)
- [Diagnostics](/docs/compiler/diagnostics)
- [Compilation pipeline](/docs/compiler/compilation-pipeline)
