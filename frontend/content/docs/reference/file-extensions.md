The accepted source suffixes are `.prnc` and `.princi`. They are interchangeable aliases for exactly the same Princi language.

| Input | Default output |
| --- | --- |
| `hello.prnc` | `hello.exe` |
| `hello.princi` | `hello.exe` |

Both extensions use the same source loader, lexer, parser, semantic rules, code generator, and Windows target. A suffix does not change syntax, types, or generated behavior.

An explicit output path overrides the derived name:

~~~powershell
princi build hello.princi -o app.exe
~~~

Other source extensions are rejected with a diagnostic.
