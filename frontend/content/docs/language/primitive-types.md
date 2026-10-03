Princi v0.1 has five primitive types:

| Type | Meaning |
| --- | --- |
| `Int` | Signed 64-bit integer in the native backend |
| `Float` | 64-bit floating-point value |
| `Bool` | `true` or `false` |
| `String` | UTF-8 string value |
| `Void` | No returned value |

Function and method return types default to `Void`. Parameters and fields require explicit non-`Void` types.

## No implicit numeric conversion

Integer and floating-point expressions do not automatically convert between `Int` and `Float`. Arithmetic and comparisons require matching numeric operand types. The compiler does not apply a common numeric type to mixed expressions or list literals.

## Nominal user types and List

Class and struct declarations introduce nominal types with their declared names. `List<T>` is the only generic source type in v0.1 and requires one non-`Void` element type. See [built-in types](/docs/reference/built-in-types) for list behavior.
