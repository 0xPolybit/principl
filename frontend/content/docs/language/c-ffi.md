The v0.1 FFI surface declares direct, non-variadic external functions inside an `extern "C"` block:

~~~princi
extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    println(abs(-42))
}
~~~

Only `Int32`, `Int64`, and `Float64` are allowed as parameters. Returns may use those types or `Void`. At Princi call sites, `Int32` and `Int64` appear as `Int`, while `Float64` appears as `Float`. Int32 arguments are checked for signed 32-bit range and results are sign-extended.

The backend uses the target's Windows x86-64 C ABI. Managed Princi values, structs, booleans, pointers, callbacks, C++ conventions, variadic functions, and user library/object linker flags are not supported.
