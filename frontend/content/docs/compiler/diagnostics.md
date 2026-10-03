Compiler errors use a file, 1-based line and column, stable diagnostic code, and source span:

~~~text
example.prnc:6:12: error[E0201]: expected Int, found String (expression)

    var age: Int = "twenty"
                   ^^^^^^^^^
~~~

## Source errors

| Code | Meaning |
| --- | --- |
| `E0002` / `E0003` | Unsupported file extension / source file error |
| `E0100` / `E0101` | Lexical error / syntax error |
| `E0201` | Type mismatch |
| `E0202` / `E0203` | Unknown identifier or function / unknown type |
| `E0204` / `E0205` | Incorrect argument count / invalid member access |
| `E0206` / `E0207` | Duplicate declaration / missing or invalid `main` |
| `E0208` / `E0209` / `E0210` | Invalid return / unsupported import / read before initialization |

## Toolchain and internal errors

`E0401` reports missing LLVM/Clang, `E0403` reports a missing or incompatible Windows linker, and `E0404` reports link failure. Generated-IR verification failures are internal errors (`E9002`); unexpected compiler failures use `E9001`. Compiler failures exit non-zero and are not presented as source errors.

Set `PRINCI_KEEP_INTERMEDIATES=1` to retain the generated IR and object when investigating a backend failure. The CLI does not silently turn diagnostics into successful builds.
