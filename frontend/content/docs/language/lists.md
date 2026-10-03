`List<T>` is Princi's first and only built-in generic collection in v0.1:

~~~princi
fn main() {
    var numbers: List<Int> = [1, 2, 3]
    numbers[1] = 7
    numbers.add(9)

    println(numbers.length)
    println(numbers[1])
}
~~~

Non-empty list literals infer one homogeneous element type. An empty list needs an expected type, such as `var values: List<Int> = []`. Heterogeneous literals are rejected; v0.1 does not infer a common element type.

## Operations

| Operation | Behavior |
| --- | --- |
| `list.length` | Read-only `Int` length |
| `list[index]` | Read an element using an `Int` index |
| `list[index] = value` | Replace an element with a value of type `T` |
| `list.add(value)` | Append a value of type `T` |

Generated code checks both negative and past-end indexes at runtime. Lists are heap-backed references, so copies share contents. A `let` list binding cannot be replaced but its elements and length may be mutated through the supported operations. Removal, iteration APIs, and other collection types are not implemented.
