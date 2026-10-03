PrinciPL (usually shortened to Princi) is a statically typed programming language with a compiler written in Rust. The v0.1 compiler produces native Windows x86-64 executables.

The language combines a compact procedural core with reference-oriented classes, value-oriented structs, and one built-in generic collection, `List<T>`. Local variables can infer their type from an initializer; function parameters and fields are explicitly typed.

> **Scope note**
> This guide describes the compiler that exists in the repository today. The native backend supports the documented procedural, class, struct, string, and list subset. Features named as future direction are not available in v0.1.

## Build a first program

Save this as `hello.prnc` or `hello.princi`:

~~~princi
fn main() {
    println("Hello from Princi!")
}
~~~

Both suffixes name the same source language. Build either one with `princi build <source-file>`; the default output is `hello.exe` beside the source.

## Where to go next

- [Install the compiler and Windows toolchain](/docs/getting-started/installation).
- [Build and run Hello, world](/docs/getting-started/hello-world).
- [Read the supported syntax and type rules](/docs/language/syntax).
- [Review the v0.1 limitations](/docs/reference/limitations).

The repository [README](https://github.com/0xPolybit/principl/blob/main/README.md) and [v0.1 scope document](https://github.com/0xPolybit/principl/blob/main/docs/v0.1-scope.md) remain the detailed project references.
