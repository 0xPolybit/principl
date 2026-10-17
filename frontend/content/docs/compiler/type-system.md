Princi's source type system is implemented in `types.rs` and checked by semantic analysis. Types are nominal for user-declared classes, structs, enums, interfaces, and traits. Generic declarations carry parameter identities; concrete generic uses carry their type arguments. The backend receives resolved types rather than interpreting type names from raw source text.

## Source types

The source language provides `Int`, `Float`, `Bool`, `String`, and `Void`, plus declared nominal types and `List<T>`. User-declared generic functions, classes, structs, and enums can take type parameters; each declaration enforces its arity and bounds.

~~~princi
struct Point {
    x: Float
    y: Float
}

fn describe(points: List<Point>, enabled: Bool) -> Int {
    if enabled {
        return points.length
    }
    return 0
}

fn main() {
    let sample = [Point(1.0, 2.0)]
    println(describe(sample, true))
}
~~~

## Generic and dynamic types

The type model distinguishes concrete types, generic parameters, and instantiated generic types. `List<T>` uses the general generic type representation while keeping its existing runtime storage and compiler intrinsics. Generic code is monomorphized for concrete specializations. Generic parameter bounds can require declared traits or interfaces. Interface and trait values retain a data reference plus a dispatch-table reference; interface conversion is supported for classes, while a struct-to-trait conversion boxes a copy.

## Internal type representation

The compiler's `Type` representation covers primitives, nominal types, generic parameters and instantiations, `List`, interface/trait values, and internal compiler-only types. `Any` is used narrowly for the type-checked built-ins `print` and `println`; it is not a source-level dynamic type. `Error` allows analysis to continue after an earlier failure without generating cascaded type errors.

`FunctionType` stores an ordered parameter type list and a return type. `Type::is_managed_reference` currently identifies `String`, class references, and `List<T>`; structs are inline values even if one of their fields contains a managed reference.

## FFI types are separate

The `ffi` module has its own `CAbiType` for `Int32`, `Int64`, `Float64`, and `Void`. Those names are accepted only inside `extern "C"` declarations. At Princi call sites they map to `Int`, `Int`, `Float`, and `Void`, while the separate ABI signature preserves the widths needed for conversions.

## Type checking rules

Assignments, calls, returns, list elements, fields, and operators must match expected types. `Int` and `Float` are not implicitly converted. Class and list values are managed references; struct values are copied inline. The source-level type rules are described in the [language type guide](/docs/language/primitive-types).

## Current v0.2 limitations

There is no `Any` syntax, union or nullable type, pointer type, or user-defined cast. `Function`, `Range`, `Any`, and `Error` are compiler-side representations, not types that ordinary source declarations may name. Generic methods, generic traits/interfaces, associated types, and advanced generic mechanisms remain unsupported.

## Related topics

- [Semantic analysis](/docs/compiler/semantic-analysis)
- [Primitive types](/docs/language/primitive-types)
- [Lists](/docs/language/lists)
- [C FFI boundary](/docs/compiler/c-ffi-boundary)
