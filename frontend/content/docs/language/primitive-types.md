Princi v0.1 has five primitive types. A variable annotation, parameter, field, or return declaration names a type directly.

| Type | Source values | Current native representation |
| --- | --- | --- |
| `Int` | Whole signed integers | Signed 64-bit integer (`i64`) |
| `Float` | Floating-point numbers | 64-bit IEEE floating-point (`double`) |
| `Bool` | `true`, `false` | LLVM `i1` boolean value |
| `String` | UTF-8 text | Runtime-managed pointer to NUL-terminated UTF-8 storage |
| `Void` | No value | No result value |

~~~princi
fn describe(count: Int, ratio: Float, active: Bool, label: String) {
    println(label)
    println(count)
    println(ratio)
    println(active)
}

fn main() {
    describe(3, 0.75, true, "ready")
}
~~~

## Numeric types

`Int` is signed 64-bit in the native backend. Integer arithmetic wraps on 64-bit overflow. `Float` is 64-bit floating point. Numeric expressions require matching operand types; Princi does not implicitly convert between `Int` and `Float`.

~~~princi
fn main() {
    let whole: Int = 12
    let fractional: Float = 12.5
    println(whole)
    println(fractional)
    println(whole < 20)
}
~~~

## Boolean and strings

Boolean literals are the lowercase words `true` and `false`. Conditions and boolean operators require `Bool`. Strings are double-quoted UTF-8 values; see [Strings](/docs/language/strings) for escapes and storage behavior.

~~~princi
fn main() {
    let is_ready: Bool = true
    let message: String = "build complete"
    if is_ready {
        println(message)
    }
}
~~~

## Void

When a function or method omits `-> Type`, its return type is `Void`. It may use `return` without a value or reach the end. A non-`Void` function must return a matching value along every control-flow path.

~~~princi
fn log_count(value: Int) {
    println(value)
    return
}

fn answer() -> Int {
    return 42
}

fn main() {
    log_count(answer())
}
~~~

`Void` is not valid for parameters, fields, list elements, or local value inference.

## Current v0.1 limitations

There are no unsigned integer, smaller integer, decimal, character, or nullable types. Numeric conversions must be explicit only if a future language version defines them; v0.1 has no cast syntax. String storage rejects embedded NUL bytes. See [C interoperability](/docs/language/c-ffi) for the separate FFI-only integer and float widths.

## Related topics

- [Variables](/docs/language/variables)
- [Operators](/docs/language/operators)
- [Functions](/docs/language/functions)
- [C interoperability](/docs/language/c-ffi)
