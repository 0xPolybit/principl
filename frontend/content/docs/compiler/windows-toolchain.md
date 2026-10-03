Native compilation in v0.1 is supported only on Windows x86-64. Install:

1. Stable Rust and Cargo using [rustup](https://rust-lang.org/tools/install/).
2. LLVM/Clang 15 or newer with LLVM IR input support and the X86 backend.
3. x86-64 MinGW-w64 GCC, commonly provided by the MSYS2 UCRT64 environment.

## Check the target tools

~~~powershell
clang --version
clang --print-targets
gcc -dumpmachine
~~~

The target list must include X86 and GCC should report `x86_64-w64-mingw32`. Add Clang's `bin` directory and MSYS2's `ucrt64\bin` directory to Windows `PATH`.

The compiler reads `clang` and `gcc` from `PATH` by default. Use `PRINCI_CLANG` or `PRINCI_CC` to select alternate executables. Missing tools, incompatible targets, and link failures receive separate diagnostics.

The compiler explicitly targets `x86_64-w64-windows-gnu`; host defaults do not select a different output platform. Installation walkthrough: [Windows setup in the repository README](https://github.com/0xPolybit/principl#windows-setup).
