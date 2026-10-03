The global prelude provides two built-ins:

| Function | Accepted value | Output |
| --- | --- | --- |
| `print(value)` | One non-`Void` primitive: `Int`, `Float`, `Bool`, or `String` | Writes without a trailing newline |
| `println(value)` | Same primitive values | Writes a newline after the value |

The compiler selects a type-specific runtime formatter after semantic analysis. Internal symbols such as `princi_rt_print_int` are implementation details and are not callable from Princi source.

Lists, classes, structs, and `Void` values are not accepted as print arguments in v0.1.
