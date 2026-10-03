The v0.1 compiler exposes one user-facing command:

```text
princi build <source-file> [-o <output-file>]
```

The source must have the `.prnc` or `.princi` extension. Those extensions are exact aliases for the same Princi source language.

## Build with the default output name

From PowerShell, in the source directory:

```powershell
princi build hello.prnc
```

The compiler writes `hello.exe` beside `hello.prnc`. The extension does not change the output rule:

```powershell
princi build hello.princi
```

This also writes `hello.exe` beside the input. Existing output files with the same path are replaced only after the new executable has linked successfully.

## Choose an output path

Pass `-o` followed by an executable path:

```powershell
princi build hello.prnc -o app.exe
```

This creates `app.exe`. A relative output path is resolved from the current working directory, not the directory containing the source. You can also pass a nested or absolute path:

```powershell
princi build src\hello.princi -o build\app.exe
princi build src\hello.prnc -o "C:\Users\Public\Princi builds\app.exe"
```

Quote paths with spaces according to PowerShell syntax. Ensure the destination directory exists and is writable.

## Run the output

The build command compiles; it does not launch the result. Run a relative executable from PowerShell with `.` and a backslash:

```powershell
.\hello.exe
.\app.exe
```

The produced file is a Windows x86-64 executable. A failed compilation or link exits with a non-zero status and does not leave a partial executable at the requested output path. The CLI does not include `run`, `test`, `fmt`, `package`, `install`, or `repl` subcommands.

## What happens during a build

```text
.prnc / .princi
  → source loading
  → lexer
  → parser and AST
  → built-in module resolution
  → semantic analysis and typed representation
  → LLVM IR verification and Windows COFF object generation
  → MinGW-w64 link
  → Windows x86-64 .exe
```

Clang verifies the generated LLVM IR and emits an object for `x86_64-w64-windows-gnu`. MinGW-w64 GCC, which must report `x86_64-w64-mingw32`, links the object with the Windows C runtime. Temporary LLVM IR and object files are removed after the build by default; compiler development can retain them with `PRINCI_KEEP_INTERMEDIATES=1`.

For installation and PATH troubleshooting, see [Installation](/docs/installation). For stage-by-stage details, see the [compilation pipeline](/docs/compiler/compilation-pipeline).
