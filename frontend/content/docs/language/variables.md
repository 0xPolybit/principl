Function-local bindings use `let` or `var`:

~~~princi
fn main() {
    let pi: Float = 3.14159
    var count = 10
    count += 1
    println(count)
}
~~~

`let` creates an immutable binding and requires an initializer. `var` creates a mutable binding. It may be declared without an initializer only when an explicit type is present; the value must be assigned before it is read.

## Inference and annotations

An initializer gives an unannotated local its type:

~~~princi
var count = 10        // Int
var name = "Octrie"   // String
~~~

Inference is local and predictable. It does not infer types across declarations or perform numeric promotion. An annotation can be used with or without an initializer:

~~~princi
var count: Int = 10
var later: Float
let enabled: Bool = true
~~~

Parameters and `for` loop variables are immutable. A nested block may shadow a name from an outer scope, but duplicate declarations in the same scope are errors. A binding cannot be reassigned through a `let` name.
