Princi uses `if`/`else`, `while`, and integer range `for` statements to control execution. Blocks are written with braces. `if` and `while` conditions must evaluate to `Bool`.

## If and else

An `if` may have no `else`, a block `else`, or an `else if` chain:

~~~princi
fn describe(score: Int) {
    if score >= 90 {
        println("high")
    } else if score >= 50 {
        println("passing")
    } else {
        println("keep going")
    }
}

fn main() {
    describe(72)
}
~~~

Conditions are not implicitly converted from numbers or other types. Use comparisons or boolean expressions to produce a `Bool`.

## While

`while` checks its condition before each iteration, so its body may execute zero times:

~~~princi
fn main() {
    var count = 0
    while count < 3 {
        println(count)
        count += 1
    }
}
~~~

## Return and branch coverage

`return` exits the current function. A non-`Void` function must provide a value of its declared return type on every possible path:

~~~princi
fn absolute(value: Int) -> Int {
    if value < 0 {
        return -value
    } else {
        return value
    }
}

fn main() {
    println(absolute(-4))
}
~~~

Definite initialization follows the same branch rule: assigning a local in both the `if` and `else` branches initializes it afterward. With no `else`, the compiler cannot assume the branch ran.

## Current v0.1 limitations

Conditions must be `Bool`. There are no `break`, `continue`, `switch`, `match`, pattern matching, or exception-handling statements. A loop body does not guarantee that a local assigned only inside it is initialized after the loop.

See [Loops](/docs/language/loops) for range behavior and [Functions](/docs/language/functions) for return rules.

## Related topics

- [Variables and definite initialization](/docs/language/variables)
- [Loops](/docs/language/loops)
- [Operators](/docs/language/operators)
- [Functions](/docs/language/functions)
