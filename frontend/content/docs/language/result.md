`Result<T, E>` is an ordinary generic enum supplied by Princi's embedded source prelude. It represents either a success payload `T` or an error payload `E`:

~~~princi
fn divide(a: Int, b: Int) -> Result<Int, String> {
    if b == 0 {
        return Result<Int, String>.Err("division by zero")
    }
    return Result<Int, String>.Ok(a / b)
}

fn main() {
    match divide(10, 2) {
        Ok(value) => println(value)
        Err(error) => println(error)
    }
}
~~~

Construct variants with the explicit generic type, for example `Result<Int, String>.Ok(42)`. In a match on a known `Result`, `Ok(value)` and `Err(error)` are contextual patterns and do not need to repeat type arguments.

## Propagate errors with `?`

The postfix `?` evaluates its operand once. On `Ok`, it yields the success value. On `Err`, it returns that error from the enclosing function immediately:

~~~princi
fn calculate(value: Int) -> Result<Int, String> {
    if value < 0 {
        return Result<Int, String>.Err("must be positive")
    }
    return Result<Int, String>.Ok(value * 2)
}

fn run() -> Result<Int, String> {
    let doubled = calculate(21)?
    return Result<Int, String>.Ok(doubled)
}
~~~

The enclosing function must return `Result<R, E>` with exactly the same error type `E`. No automatic error conversion is performed. In a local initializer, argument, or larger expression, `?` produces the success payload. Returning the whole expression `return value?` is also supported when its success type matches the enclosing function's `R`; the compiler wraps that success in `Ok`.

## Current v0.2 limitations

`Result` is not a compiler keyword or a special backend type. A top-level declaration named `Result` takes precedence over the prelude type. There is no `From`/`Into` conversion, exception handling, or propagation from non-`Result` values. The `main` entry point remains `Void` or `Int`, not `Result`.

## Related topics

- [Enums](/docs/language/enums)
- [Pattern matching](/docs/language/match)
- [Functions](/docs/language/functions)
- [Generic functions](/docs/language/generics)
