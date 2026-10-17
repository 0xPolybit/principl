Princi provides primitive `Int`, `Float`, `Bool`, `String`, and `Void` types. User declarations add nominal class, struct, enum, interface, and trait types. Generic declarations are parameterized and instantiated with type arguments, such as `Box<Int>` or `Result<Int, String>`.

Native `Int` is signed 64-bit, and `Float` is 64-bit floating point. Strings, classes, lists, and interface/trait views use managed references. Structs and enum values are inline values; their fields or payloads may contain managed references.

## Lists

`List<T>` is the built-in generic collection and uses the same generic type syntax as user-declared types. It accepts one non-`Void` element type and supports homogeneous literals, `.length`, integer indexing and indexed assignment, `.add(value)`, and runtime bounds checks. Elements can be primitives, classes, structs, enums, nested lists, or supported interface/trait values.

Non-empty list literals infer a homogeneous element type. Empty literals require an expected type:

~~~princi
fn main() {
    var numbers: List<Int> = []
    numbers.add(42)
    println(numbers[0])
}
~~~

Generic user declarations and `List<T>` both use compile-time type checking. User generic code is monomorphized for concrete types; the list runtime keeps its existing element storage and compiler-supported operations.

## Current v0.2 limitations

There is no source-level `Any`, union or nullable type, pointer type, user-defined cast, or user-defined memory layout. `Function`, `Range`, `Any`, and `Error` are compiler-side representations, not types ordinary source declarations may name.

## Related topics

- [Primitive types](/docs/language/primitive-types)
- [Generics](/docs/language/generics)
- [Lists](/docs/language/lists)
- [Type system internals](/docs/compiler/type-system)
