The lexer recognizes identifiers beginning with a Unicode letter or underscore and continuing with letters, digits, or underscores.

## Literals and comments

- Decimal integer and floating-point literals, including exponents.
- Double-quoted strings with `\\`, `\"`, `\n`, `\r`, and `\t` escapes.
- Boolean literals `true` and `false`.
- `//` line comments and whitespace.

NUL bytes and `\0` escapes are rejected. The keywords are `fn`, `return`, `let`, `var`, `if`, `else`, `while`, `for`, `in`, `class`, `struct`, `init`, `self`, `import`, and `extern`.

## Operators and punctuation

Operators: `+ - * / % = == != < <= > >= && || ! += -= *=`.

Punctuation: `( ) { } [ ] , . : ; -> ..`.

The lexer preserves semicolons. Newline and semicolon handling at statement boundaries is a parser rule: semicolons may be used where grammar permits, while otherwise a line break can separate statements. Each token retains a file, 1-based line and character column, and end-exclusive UTF-8 byte span.
