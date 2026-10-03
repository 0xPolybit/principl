Local variables live inside function, method, or nested block scopes. Use `let` for a binding that cannot be reassigned and `var` for a binding that can be reassigned.

## Declare a value

~~~princi
fn main() {
    let pi: Float = 3.14159
    var count = 10
    count += 1
    println(pi)
    println(count)
}
~~~

An annotation has the form `name: Type`. If a declaration has an initializer and no annotation, its type is inferred from that initializer:

~~~princi
fn main() {
    var score = 42          // Int
    let title = "PrinciPL"  // String
    var ready = true        // Bool
    println(score)
    println(title)
    println(ready)
}
~~~

This is local, direct inference; it does not promote numbers or infer a type by looking at later assignments. An annotation can be paired with an initializer to require an exact type:

~~~princi
fn main() {
    var count: Int = 10
    let scale: Float = 0.5
    println(count)
    println(scale)
}
~~~

`Int` and `Float` do not implicitly convert. For example, assigning an `Int` initializer to a `Float` annotation is a type error.

## Mutability

`let` is immutable and must have an initializer. `var` is mutable and may be declared first with an explicit type, then initialized later:

~~~princi
fn main() {
    var result: Int
    result = 6 * 7
    println(result)
}
~~~

Assigning to a `let` binding is rejected. Function parameters and `for` loop variables are immutable too. A nested block can shadow an outer local; duplicate names in the same scope are errors.

Binding mutability is separate from mutability of a referenced value. A `let` list cannot be rebound, but its elements may be updated with supported list operations. See [Lists](/docs/language/lists).

## Definite initialization

A local must be assigned before it is read. An assignment in both arms of an `if`/`else` guarantees a value after the branch:

~~~princi
fn choose(flag: Bool) -> Int {
    var result: Int
    if flag {
        result = 1
    } else {
        result = 2
    }
    return result
}
~~~

An assignment only inside a loop does not guarantee initialization after the loop, because the loop can execute zero times.

## Current v0.1 limitations

Inference only uses a declaration's initializer. There is no flow-sensitive type inference, implicit numeric conversion, destructuring, global mutable variable syntax, or reassignment of immutable bindings.

## Related topics

- [Primitive types](/docs/language/primitive-types)
- [Functions and immutable parameters](/docs/language/functions)
- [Control flow and initialization](/docs/language/control-flow)
- [Lists and reference mutation](/docs/language/lists)
