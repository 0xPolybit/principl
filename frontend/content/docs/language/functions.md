Functions have typed parameters, an optional return annotation, and a block body:

~~~princi
fn add(left: Int, right: Int) -> Int {
    return left + right
}

fn main() {
    println(add(20, 22))
}
~~~

When the return annotation is omitted, the result is `Void`. Parameters are immutable. Calls require the declared argument count and matching types.

## Recursion

Functions can call themselves:

~~~princi
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }

    return n * factorial(n - 1)
}
~~~

The required entry point is a top-level `main` with no parameters. It may return `Void` or `Int`; an `Int` result becomes the process exit status. Methods are declared inside classes and use static dispatch.
