Princi v0.1 can declare direct external functions that use the Windows x86-64 C ABI. Keep declarations inside an `extern "C"` block and give each function a C-safe signature:

~~~princi
extern "C" {
    fn abs(value: Int32) -> Int32
    fn native_count() -> Int64
    fn native_ratio(value: Float64) -> Float64
    fn native_reset()
}

fn main() {
    println(abs(-42))
}
~~~

Only `Int32`, `Int64`, and `Float64` parameters are supported. Return types may be one of those or `Void`; an omitted return type also means `Void`. `Void` is not a valid parameter type.

## Mapping at call sites

Within Princi, `Int32` and `Int64` parameters/results appear as `Int`, while `Float64` appears as `Float`. An `Int32` argument is checked to fit signed 32-bit range before the call; an `Int32` result is widened to Princi's `Int`. `Int64` uses Princi's 64-bit `Int`; `Float64` uses its 64-bit `Float`.

~~~princi
extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    let magnitude: Int = abs(-17)
    println(magnitude)
}
~~~

The compiler emits the Windows x86-64 C calling convention. A declared symbol must be provided by the linked Windows toolchain/runtime; the v0.1 CLI does not accept extra library or object-file linker flags.

## Current v0.1 limitations

This is a narrow ABI boundary, not general pointer interoperability. Parameters and results cannot use `Bool`, `String`, classes, structs, lists, aggregates, or Princi-managed references. There are no raw pointer types, callbacks, variadic declarations, C++ conventions, alternate calling conventions, `repr(C)` layouts, manual memory operations, or pinning. External declarations have no Princi body and cannot overload another declared external name.

## Related topics

- [Primitive types](/docs/language/primitive-types)
- [Windows toolchain](/docs/compiler/windows-toolchain)
- [Diagnostics](/docs/compiler/diagnostics)
- [Explicit v0.1 scope](/docs/reference/limitations)
