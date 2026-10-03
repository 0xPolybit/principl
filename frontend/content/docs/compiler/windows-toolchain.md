Princi v0.1 produces Windows x86-64 executables using LLVM/Clang and x86-64 MinGW-w64 GCC. The compiler invokes these tools with structured process arguments and an explicit target; it does not construct a shell command from paths.

## Object and link steps

After creating the LLVM IR, the compiler runs the equivalent of:

~~~text
clang -x ir -target x86_64-w64-windows-gnu -c program.ll -o program.obj
~~~

Clang's IR reader checks the module while emitting a Windows COFF object. The compiler then queries GCC with `-dumpmachine`; the reported target must begin with `x86_64-w64-mingw32`. It links the object with `gcc -m64` and writes the resulting `.exe` to a staged output path beside the requested destination. Once linking succeeds, the staged executable is moved to the final path.

The Windows runtime helper definitions are included in the LLVM module. MinGW-w64 GCC supplies the normal Windows C runtime link. Users do not need to build or manually link Princi runtime files.

## Required tools

- LLVM/Clang 15 or newer, with LLVM IR input support and the X86 backend.
- x86-64 MinGW-w64 GCC, for example from MSYS2 UCRT64.
- Both tool directories on Windows `PATH`, unless the compiler executable overrides below are used.

~~~powershell
gcc -dumpmachine
~~~

The LLVM setup can be checked with `clang --version` and `clang --print-targets`; the target list must include X86. GCC should report `x86_64-w64-mingw32`.

## Tool selection and temporary files

By default the compiler starts `clang` and `gcc` by name from `PATH`. Set `PRINCI_CLANG` or `PRINCI_CC` to select a specific executable. LLVM IR and object files are created in a unique temporary build directory and removed when compilation finishes. Set `PRINCI_KEEP_INTERMEDIATES=1` during compiler development to keep those files and print their location.

`princi build` hides intermediate files from the normal user workflow. The source extension does not affect the selected target or linker path.

## Toolchain diagnostics

`E0401` means LLVM/Clang could not be started; `E0403` means the linker is unavailable or targets the wrong architecture; `E0404` reports a linker command failure. LLVM/native generation failure is reported separately as `E9002`. These errors exit nonzero rather than producing a success message.

## Current v0.1 limitations

Only the Windows x86-64 GNU target is supported. The compiler does not target MSVC, Linux, macOS, ARM, or WebAssembly. It does not provide CLI flags for additional library or object files. See [Installation](/docs/installation) for the complete Windows setup.

## Related topics

- [Compilation pipeline](/docs/compiler/compilation-pipeline)
- [LLVM backend](/docs/compiler/llvm-backend)
- [Runtime](/docs/compiler/runtime)
- [C FFI boundary](/docs/compiler/c-ffi-boundary)
