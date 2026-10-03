Functions are top-level named declarations. Parameters require explicit types. Add `-> Type` to return a value; when omitted, the return type is `Void`.

~~~princi
fn add(left: Int, right: Int) -> Int {
    return left + right
}

fn main() {
    let total = add(20, 22)
    println(total)
}
~~~

Calls must provide the declared number of arguments, in order, with matching types. The compiler checks calls before generating native code.

## Return values and Void

A `Void` function can reach the end or use a bare `return`. It cannot return a value. A non-`Void` function must return a correctly typed value on every control-flow path.

~~~princi
fn announce(message: String) {
    println(message)
}

fn double(value: Int) -> Int {
    return value * 2
}

fn main() {
    announce("ready")
    println(double(21))
}
~~~

Parameters are immutable bindings. A function can create mutable local copies with `var` if it needs to change a value.

## Recursion

Functions may call themselves:

~~~princi
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

fn main() {
    println(factorial(5))
}
~~~

## Program entry point

Every program needs a top-level `main` function with no parameters. It may return `Void` or `Int`. If it returns `Int`, that value becomes the Windows process exit status.

~~~princi
fn main() -> Int {
    println("complete")
    return 0
}
~~~

See [methods](/docs/language/methods) for functions attached to class instances.

## Current v0.1 limitations

There are no closures, function overloading, default or named arguments, variadic functions, function literals, or generic functions. Calls use positional arguments. Return type inference is not provided.

## Related topics

- [Variables and parameter mutability](/docs/language/variables)
- [Primitive types](/docs/language/primitive-types)
- [Control flow](/docs/language/control-flow)
- [Methods](/docs/language/methods)
