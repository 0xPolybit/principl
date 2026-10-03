The v0.1 command-line interface has one command:

~~~text
princi build <source-file> [-o <output-file>]
~~~

## Default output

~~~powershell
princi build hello.prnc
~~~

The compiler derives `hello.exe` from the input's filename and writes it beside the source. The same behavior applies to `hello.princi`.

## Explicit output

~~~powershell
princi build src\hello.princi -o out\app.exe
~~~

The `-o` option selects the executable path. Paths may contain spaces when they are quoted according to the shell:

~~~powershell
princi build "examples\first program.prnc" -o "build output\first.exe"
~~~

The compiler parses and checks the program, verifies generated LLVM IR, emits a Windows object, and links the final executable. Intermediate files are normally kept in a temporary build directory and removed. Set `PRINCI_KEEP_INTERMEDIATES=1` when debugging the compiler to retain them.

No `run`, `test`, `fmt`, package, installation, or REPL command is part of the v0.1 CLI.
