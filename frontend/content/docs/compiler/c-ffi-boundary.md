The compiler keeps C ABI declarations separate from ordinary Princi values. The source syntax is `extern "C"`; semantic analysis resolves a narrow set of ABI-safe scalar types, and the backend emits direct calls using the Windows x86-64 C calling convention.

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

Only `Int32`, `Int64`, and `Float64` parameters are accepted. Return types may be those three or `Void`; omitting `-> ReturnType` also means `Void`. `Void` cannot be a parameter. External declarations have no Princi body.

## Type mapping

| C declaration type | Princi type at call site | Boundary handling |
| --- | --- | --- |
| `Int32` | `Int` | Argument range-checked to signed 32-bit; result sign-extended to `Int` |
| `Int64` | `Int` | Signed 64-bit integer |
| `Float64` | `Float` | C `double` |
| `Void` | `Void` return only | No result value |

~~~princi
extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    let magnitude: Int = abs(-17)
    println(magnitude)
}
~~~

The symbol is called directly. The linker must be able to resolve it through the normal configured Windows link; the Princi CLI does not accept extra library or object-file flags.

## Safety boundary

The C ABI model lives in `ffi.rs` as `CAbiType` and `ExternalFunctionType`, separate from `types::Type`. This keeps the foreign width (`i32`, `i64`, or `double`) even though Princi expressions see `Int` or `Float`. FFI types are rejected in ordinary source declarations.

~~~princi
extern "C" {
    fn pass_text(value: String) -> String
}
~~~

The example above is rejected: managed Princi strings cannot cross the v0.1 C boundary. `Bool`, classes, structs, lists, pointers, and other aggregates are likewise unsupported.

## Current v0.1 limitations

Calls are direct and non-variadic under the Windows x86-64 C ABI. The compiler does not import C headers or compile C sources. There are no callbacks, C++ or alternate calling conventions, raw pointers, `repr(C)` structs, aggregate arguments, manual allocation/free, or pinning. External names must satisfy the restricted ASCII C identifier rules and cannot collide with compiler/runtime symbols.

## Related topics

- [Type system](/docs/compiler/type-system)
- [Windows linking](/docs/compiler/windows-toolchain)
- [C interoperability language syntax](/docs/language/c-ffi)
- [Diagnostics](/docs/compiler/diagnostics)
