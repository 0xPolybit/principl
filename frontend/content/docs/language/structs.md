Structs are field-only value types. They have typed fields, no methods, and no `init` constructor:

~~~princi
struct Point {
    x: Float
    y: Float
}

fn shifted(point: Point) -> Point {
    return Point(point.x + 1.0, point.y)
}
~~~

Positional construction follows field declaration order. Named construction with `Point { x: ..., y: ... }` requires every field exactly once. Struct values copy when assigned, passed as function arguments, or returned. A mutable `var` may update a field; immutable bindings and parameters may not.

Struct layouts follow declaration order and are stored inline. A struct field containing a class or list copies that managed reference, preserving the referenced value's sharing behavior. Recursive by-value struct fields are rejected because they have no finite layout.
