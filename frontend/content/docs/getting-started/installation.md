The Princi compiler targets Windows x86-64. Native builds need Rust/Cargo to build the compiler, LLVM/Clang to verify IR and emit a Windows object, and MinGW-w64 GCC to link the executable.

## Prerequisites

| Tool | Requirement | Check |
| --- | --- | --- |
| Rust stable and Cargo | Current stable toolchain | `rustc --version` |
| LLVM/Clang | Clang 15 or newer, with LLVM IR input and the X86 target | `clang --version` and `clang --print-targets` |
| MinGW-w64 GCC | x86-64 Windows GNU target | `gcc -dumpmachine` |

The compiler expects the MinGW target triple `x86_64-w64-mingw32`. Follow the Windows setup steps in the [repository README](https://github.com/0xPolybit/principl#windows-setup) for installation guidance.

## Build the compiler

From a clean clone, build a debug or release executable with Cargo:

~~~powershell
cargo build
cargo build --release
~~~

The CLI is written to `target\debug\princi.exe` or `target\release\princi.exe`. To install the release CLI into Cargo's user binary directory:

~~~powershell
cargo install --path .
~~~

## Verify the tools

~~~powershell
rustc --version
clang --version
clang --print-targets
gcc -dumpmachine
~~~

If Clang, its X86 backend, or the correct MinGW linker is missing, the compiler reports a toolchain diagnostic during a build.
