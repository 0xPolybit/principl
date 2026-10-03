Princi uses explicit blocks with braces. Newlines commonly separate statements, so semicolons are optional where the parser can identify a statement boundary. A newline after an operator continues the expression.

## Top-level declarations

The parser accepts functions, classes, structs, built-in imports, and restricted `extern "C"` blocks:

~~~princi
import io

fn square(value: Int) -> Int {
    return value * value
}
~~~

Imports resolve only against the built-in registry. The native backend currently compiles the procedural, class, field-only struct, string, and List<T> subset.

## Statements

Function bodies support `let` and `var` declarations, assignments, nested blocks, `if`/`else`, `while`, integer range `for` loops, `return`, and expression statements.

## Expressions

Expressions include primitive literals, names, `self`, list literals, named-field construction, function and method calls, member access, indexes, ranges, unary operators, and binary operators.

Binary operators associate left to right. Precedence, from low to high:

| Level | Operators |
| --- | --- |
| 1 | `..` |
| 2 | `||` |
| 3 | `&&` |
| 4 | `==`, `!=` |
| 5 | `<`, `<=`, `>`, `>=` |
| 6 | `+`, `-` |
| 7 | `*`, `/`, `%` |
| 8 | Unary `+`, `-`, `!` |
| 9 | Calls, member access, indexes |

Ranges are non-associative unless parenthesized. See [lexical syntax](/docs/reference/lexical-syntax) for the token boundary.
