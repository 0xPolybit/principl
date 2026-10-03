This guide takes you from a clean Windows machine to a running Princi executable. Princi v0.1 compiles native Windows x86-64 programs; the compiler is written in Rust and uses LLVM/Clang plus MinGW-w64 for native output.

## Follow these steps

1. [Install the prerequisites and build the compiler](/docs/installation).
2. [Create and run a Hello, world program](/docs/hello-world).
3. [Learn the build command, output naming, and `-o`](/docs/compiling).

## You will need

- Windows x86-64.
- The stable Rust toolchain and Cargo to build the compiler.
- LLVM/Clang 15 or newer with LLVM IR input and the X86 target.
- x86-64 MinGW-w64 GCC to link the generated Windows object into an executable.

> **Windows-specific:** The generated program targets Windows x86-64. Installing the Rust compiler alone is not enough to build a Princi program; Clang and the MinGW-w64 linker must also be available.

## Build and install choices

From the repository root, `cargo build` creates a debug compiler at `target\debug\princi.exe`; `cargo build --release` creates the optimized compiler at `target\release\princi.exe`. To install the release command into Cargo's user binary directory, run `cargo install --path .` from the cloned repository. The directory `%USERPROFILE%\.cargo\bin` must be on `PATH` for the `princi` command to be found in a new terminal.

## Source file names

Save Princi source as either `.prnc` or `.princi`. The compiler treats the extensions as identical aliases. For example, both `princi build hello.prnc` and `princi build hello.princi` produce `hello.exe` when the input filename is `hello`.
