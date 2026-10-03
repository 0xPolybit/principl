The v0.1 CLI exposes only the build command:

~~~text
princi build <source-file> [-o <output-file>]
~~~

## Examples

~~~powershell
princi build hello.prnc
princi build hello.princi
princi build hello.prnc -o app.exe
~~~

Without `-o`, the compiler derives the `.exe` filename from the input filename and writes it beside the source. The `-o` argument selects the output path.

Unsupported extensions and missing source files produce diagnostics. A failed compilation returns a non-zero exit code and does not report success. There are no other v0.1 commands or compiler flags.
