`String` stores UTF-8 text. Write string literals with double quotes and use `+` to concatenate two strings.

~~~princi
fn greeting(name: String) -> String {
    return "Hello, " + name + "!"
}

fn main() {
    let message = greeting("Mira")
    print(message)
    println(" Welcome.")
}
~~~

`print` writes without a line break; `println` adds one. Both accept `String` as well as the other non-`Void` primitive types.

## Escapes

The lexer supports these escapes in double-quoted literals:

| Escape | Value |
| --- | --- |
| `\\` | Backslash |
| `\"` | Double quote |
| `\n` | Line feed |
| `\r` | Carriage return |
| `\t` | Tab |

~~~princi
fn main() {
    println("first line\nsecond line")
    println("say \"hello\"")
    println("path: C:\\tools")
}
~~~

String equality compares text content, not whether two expressions point to the same storage:

~~~princi
fn main() {
    let left = "native"
    let right = "nat" + "ive"
    println(left == right)
}
~~~

String literals are immutable module constants. Concatenation allocates a managed result. String values can be passed to and returned from functions like other values.

## Current v0.1 limitations

Embedded NUL bytes, including a `\0` escape, are rejected because runtime strings use NUL-terminated storage. There is no interpolation, mutable character indexing, slicing, formatting syntax, or implicit conversion from other types. Strings cannot cross the current C FFI boundary.

The runtime keeps dynamically allocated string data until `main` finishes, then releases the managed allocations as a group. See [managed memory](/docs/compiler/managed-memory).

## Related topics

- [Primitive types](/docs/language/primitive-types)
- [Operators](/docs/language/operators)
- [Functions](/docs/language/functions)
- [Built-in functions](/docs/reference/built-in-functions)
