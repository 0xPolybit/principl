Conditions for `if` and `while` must have type `Bool`:

~~~princi
fn main() {
    var count = 0

    if count == 0 {
        println("starting")
    } else {
        println("already running")
    }

    while count < 3 {
        count += 1
    }
}
~~~

An `if` may omit `else`. Blocks introduce lexical child scopes.

## Integer ranges

The `for` loop iterates from an inclusive start to an exclusive end:

~~~princi
for index in 0..count {
    println(index)
}
~~~

Native `for` loops require `Int` bounds. The loop variable is immutable and scoped to the loop body. A range expression outside a `for` loop accepts matching numeric bound types, but does not itself imply a loop.

## Returns

A `Void` function may use a bare `return`. Other functions must return a value matching the declared result type on every control-flow path.
