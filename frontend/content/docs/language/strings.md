String literals use double quotes and support the lexer-defined escapes `\\`, `\"`, `\n`, `\r`, and `\t`. Strings are UTF-8. Embedded NUL bytes and the `\0` escape are rejected in v0.1 because runtime strings use NUL-terminated storage.

## Concatenation and equality

The `+` operator concatenates two strings. Equality compares string contents:

~~~princi
fn main() {
    let greeting = "Hello, " + "Princi!"
    println(greeting)
    println(greeting == "Hello, Princi!")
}
~~~

Concatenation allocates a managed result. String literals are immutable constants in the generated module.

## Output

`print(value)` and `println(value)` accept primitive values, including strings. The first writes without a newline; the second appends one. Lists and `Void` values are not printable.

See [managed memory](/docs/compiler/managed-memory) for String allocation lifetime.
