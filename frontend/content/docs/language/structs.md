A struct is a nominal value type made of typed fields. It is stored inline and copied when assigned, passed to a function, or returned. Unlike a class, a struct does not have managed object identity or its own heap allocation.

~~~princi
struct Point {
    x: Float
    y: Float
}

fn distance_squared(point: Point) -> Float {
    return point.x * point.x + point.y * point.y
}

fn main() {
    let point = Point(3.0, 4.0)
    println(distance_squared(point))
}
~~~

## Construction and fields

Positional construction follows field declaration order. Named construction can list every field once in any order:

~~~princi
struct Color {
    red: Int
    green: Int
    blue: Int
}

fn main() {
    let accent = Color { blue: 180, red: 40, green: 90 }
    println(accent.green)
}
~~~

Every field must be initialized. Fields require explicit non-`Void` types. Field access reads a field value. A field can be changed through a mutable `var` struct binding:

~~~princi
struct Point {
    x: Float
    y: Float
}

fn main() {
    var point = Point(1.0, 2.0)
    point.x = 5.0
    println(point.x)
}
~~~

## Copy behavior and functions

Struct assignment, argument passing, and return values copy the struct's fields. Updating a copy does not update the original:

~~~princi
struct Point {
    x: Int
}

fn moved(value: Point) -> Point {
    var copy = value
    copy.x += 1
    return copy
}

fn main() {
    var original = Point(3)
    var result = moved(original)
    result.x += 1
    println(original.x) // 3
    println(result.x)   // 5
}
~~~

A struct field can itself contain a class reference or list handle. Copying that field copies the reference, so the referenced class or list remains shared; the enclosing struct is still copied by value.

## Current v0.1 limitations

Structs have no methods, `init` constructors, destructors, inheritance, custom layout attributes, or ownership syntax. Recursive struct fields by value are rejected because they cannot have a finite inline layout. Mutation of a struct field nested beneath a struct value read from a list index is unsupported; update the whole element and assign it back.

## Related topics

- [Classes: reference semantics](/docs/language/classes)
- [Constructors](/docs/language/constructors)
- [Functions](/docs/language/functions)
- [Lists of struct values](/docs/language/lists)
