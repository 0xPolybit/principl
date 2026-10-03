The supported binary operators are:

| Operators | Rule |
| --- | --- |
| `+`, `-`, `*`, `/` | Matching `Int` or matching `Float` operands |
| `%` | Two `Int` operands |
| `+` | Also concatenates two `String` values |
| `<`, `<=`, `>`, `>=` | Matching numeric operands |
| `==`, `!=` | Matching primitive operand types |
| `&&`, `||` | Two `Bool` operands |
| `..` | Matching numeric bounds; integer `for` loops require `Int` bounds |

Unary `+` and `-` operate on numeric values; unary `!` requires `Bool`. There are no implicit numeric conversions, so expressions such as `1 + 2.0` are rejected.

## Precedence

From lowest to highest: range, logical OR, logical AND, equality, comparisons, addition/subtraction, multiplication/division/remainder, unary operators, then calls/member access/indexing. Binary operators associate left-to-right. Range expressions are non-associative unless parentheses make the grouping explicit.
