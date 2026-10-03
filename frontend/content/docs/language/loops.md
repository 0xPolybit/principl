Princi v0.1 provides `while` loops and `for name in start..end` loops. Loop bodies are blocks, and the loop variable in a `for` loop is immutable and visible only in that loop's scope.

## While loops

A `while` loop repeats while its `Bool` condition is true. Since the condition is checked first, the body can execute zero times:

~~~princi
fn main() {
    var remaining = 3
    while remaining > 0 {
        println(remaining)
        remaining -= 1
    }
}
~~~

## Integer ranges

The range operator `..` includes the start and excludes the end. `0..4` visits `0`, `1`, `2`, and `3`:

~~~princi
fn sum_to(end: Int) -> Int {
    var total = 0
    for value in 0..end {
        total += value
    }
    return total
}

fn main() {
    println(sum_to(4)) // 6: 0 + 1 + 2 + 3
}
~~~

Bounds are evaluated as expressions, must both be `Int` in a native `for`, and the range may be empty when its start is at or past its end:

~~~princi
fn main() {
    let start = 2
    let end = 5
    for index in start..end {
        println(index)
    }
}
~~~

The `index` name is immutable. Use a separate `var` if the body needs an independently mutable counter.

## Initialization and scopes

The loop variable belongs to the loop scope and cannot be used after it. Assigning a local only inside a loop does not prove that local is initialized afterward, because the loop may execute zero times.

## Current v0.1 limitations

Native `for` loops require integer bounds and always increment by one. There is no custom step, reverse-range syntax, collection iteration, `break`, or `continue`. The range expression itself accepts matching numeric bounds, but non-integer ranges cannot drive a native `for` loop.

## Related topics

- [Control flow](/docs/language/control-flow)
- [Variables and definite initialization](/docs/language/variables)
- [Operators and range precedence](/docs/language/operators)
