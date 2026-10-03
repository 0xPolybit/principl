`List<T>` is v0.1's only generic collection. `T` is the element type, and the compiler checks each literal, read, write, and append against it.

## Create a list

A non-empty list literal infers its element type when all elements have the same type:

~~~princi
fn main() {
    var numbers = [1, 2, 3] // List<Int>
    let names: List<String> = ["Ada", "Lin"]
    println(numbers.length)
    println(names[0])
}
~~~

An empty literal has no type to infer, so provide an expected `List<T>` type:

~~~princi
fn main() {
    var pending: List<Int> = []
    pending.add(7)
    println(pending[0])
}
~~~

Mixed element types are errors. The compiler does not promote `Int` to `Float` or find a shared element type. Nested lists and lists of classes, structs, and primitive values are supported.

## Read, update, append

| Form | Behavior |
| --- | --- |
| `items.length` | Read-only `Int` length |
| `items[index]` | Read the item at an `Int` index |
| `items[index] = value` | Replace an item of type `T` |
| `items.add(value)` | Append an item of type `T`; returns `Void` |

~~~princi
fn main() {
    var scores: List<Int> = [10, 20]
    scores[1] = 25
    scores.add(30)
    println(scores.length)
    println(scores[1])
}
~~~

Indexing checks for negative and past-end positions at runtime. A failed check prints a runtime error and exits with a nonzero status rather than accessing invalid memory.

## References and mutability

Lists are heap-backed references. Assignment, parameter passing, and returns copy the list handle, so aliases see the same elements:

~~~princi
fn append_default(values: List<Int>) {
    values.add(0)
}

fn main() {
    var source = [4, 5]
    let alias = source
    append_default(alias)
    println(source.length) // 3
}
~~~

`let` prevents replacing the handle; it does not make the referenced list immutable. A list referenced by `let` may use `.add` or indexed assignment. `.length` itself cannot be assigned.

## Current v0.1 limitations

Lists do not support removal, slicing, comprehensions, iteration APIs, iterators, higher-order functions, or custom generic types. Lists cannot be passed to `print`/`println` directly. The managed runtime retains list allocations until the program exits. A struct nested inside a list is a value; to change a nested struct field, update the struct value and assign it back as a whole element.

## Related topics

- [Variables and binding mutability](/docs/language/variables)
- [Primitive types](/docs/language/primitive-types)
- [Struct values](/docs/language/structs)
- [Managed memory](/docs/compiler/managed-memory)
