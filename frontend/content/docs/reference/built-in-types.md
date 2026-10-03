The built-in source types are `Int`, `Float`, `Bool`, `String`, and `Void`. Native `Int` is signed 64-bit; `Float` is 64-bit floating point. The compiler also provides distinct nominal types for declared classes and structs.

## List<T>

`List<T>` is the only supported generic type. `T` must be one supported non-`Void` type. Lists may contain primitive, class, struct, or other list values.

Non-empty list literals infer a homogeneous element type. Empty literals require an expected type:

~~~princi
var numbers: List<Int> = []
~~~

Lists are managed reference values. Their supported operations are `length`, integer indexing for reads and writes, and `add(value)`. Runtime code checks index bounds. Other generic types and arbitrary type arguments are rejected.
