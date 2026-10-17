Princi source uses braces for blocks and a small set of top-level declarations. A source file can use either `.prnc` or `.princi`; the suffix does not change the grammar or meaning of the program.

## Top-level declarations

The v0.2 parser accepts functions, generic declarations, classes, structs, interfaces, traits, enums, built-in imports, and restricted C declarations:

~~~princi
import io

fn square(value: Int) -> Int {
    return value * value
}

struct Point {
    x: Float
    y: Float
}
~~~

Function bodies contain local declarations, assignments, branches, loops, returns, nested blocks, `match` expressions/statements, and expression statements. Classes can define fields, methods, and at most one `init`; structs contain fields and trait implementations; interfaces declare required methods, traits can include default methods, and enums declare variants.

## Statements and line breaks

Semicolons can separate statements, but a line break is normally enough:

~~~princi
fn main() {
    let first = 1
    let second = 2;
    println(first + second)
}
~~~

The parser treats a line break as a statement boundary when the next token cannot continue the preceding expression. A line break after an operator continues that expression. Braces delimit blocks; parentheses group expressions and parameter/argument lists; lists use square brackets; generic type arguments use angle brackets.

~~~princi
fn total() -> Int {
    return 10
        + 20
        * 2
}
~~~

## Expressions and precedence

Calls, member access, and indexing bind most tightly. Binary operators associate left to right; ranges are non-associative unless parentheses explicitly group them.

| Precedence, low to high | Operators |
| --- | --- |
| 1 | `..` range |
| 2 | `||` |
| 3 | `&&` |
| 4 | `==`, `!=` |
| 5 | `<`, `<=`, `>`, `>=` |
| 6 | `+`, `-` |
| 7 | `*`, `/`, `%` |
| 8 | unary `+`, `-`, `!` |
| 9 | calls, member access, indexing, postfix `?` |

~~~princi
fn main() {
    let arithmetic = 2 + 3 * 4
    let decision = true || false && false
    println(arithmetic)
    println(decision)
}
~~~

Both accepted extensions compile as the same language:

~~~text
princi build hello.prnc
princi build hello.princi
~~~

See [variables](/docs/language/variables), [operators](/docs/language/operators), and [lexical syntax](/docs/reference/lexical-syntax) for details.

## Current v0.2 limitations

There are no semantically active indentation rules, macros, decorators, or expression-valued assignments. A range cannot be chained without explicit grouping. Pattern matching supports the documented finite enum and primitive-literal subset only; parsed syntax outside the semantic or native backend subset receives a diagnostic.

## Related topics

- [Variables](/docs/language/variables)
- [Functions](/docs/language/functions)
- [Control flow](/docs/language/control-flow)
- [Operators](/docs/language/operators)
