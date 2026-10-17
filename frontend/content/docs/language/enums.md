Enums define a finite set of named variants. A variant can have no payload or a positional payload. Enum declarations can be generic.

~~~princi
enum Direction {
    North
    South
    East
    West
}

enum Maybe<T> {
    Some(T)
    None
}

fn main() {
    let direction = Direction.North
    let item = Maybe<Int>.Some(42)
    match item {
        Some(value) => println(value)
        None => println("empty")
    }
}
~~~

Write a generic enum's concrete type arguments when constructing a variant, such as `Maybe<Int>.Some(42)`. The compiler checks the variant name, payload count, and each payload type. Generic enum values work as function parameters and results and can be nested in generic classes, structs, lists, or other enums.

The native representation uses a discriminant and storage large enough for the largest declared payload. Variants and payload layouts are compiler-managed. Primitive payloads and value structs copy by value; class, string, list, and interface/trait payloads carry their corresponding managed or dispatch references.

## Current v0.2 limitations

Enums do not support C-compatible layout attributes or user-defined discriminants. Generic enum arguments are not inferred at construction. Recursive payload layouts that would require an infinitely sized inline value are rejected; indirection through reference types or lists can make a recursive shape finite.

## Related topics

- [Pattern matching](/docs/language/match)
- [Result and `?`](/docs/language/result)
- [Generics](/docs/language/generics)
