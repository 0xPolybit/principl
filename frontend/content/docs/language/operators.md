Princi checks operator operand types during semantic analysis. It does not implicitly convert between `Int` and `Float`, or choose a shared type for mixed expressions.

## Arithmetic

| Operators | Accepted operands | Result |
| --- | --- | --- |
| `+`, `-`, `*`, `/` | Two matching `Int` or two matching `Float` values | Same numeric type |
| `%` | Two `Int` values | `Int` |
| `+` | Two `String` values | `String` concatenation |

~~~princi
fn main() {
    let integer_result = 17 / 5
    let remainder = 17 % 5
    let float_result = 2.5 * 4.0
    let greeting = "Hello, " + "PrinciPL"
    println(integer_result)
    println(remainder)
    println(float_result)
    println(greeting)
}
~~~

Integer arithmetic wraps at 64 bits. There is no implicit division or numeric conversion rule that turns an `Int` expression into `Float`.

## Comparisons and equality

`<`, `<=`, `>`, and `>=` compare matching numeric values and return `Bool`. `==` and `!=` compare matching primitive values and return `Bool`; String equality compares contents.

~~~princi
fn main() {
    let within_limit = 8 <= 10
    let same_word = "compiler" == "compiler"
    let different = 1 != 2
    println(within_limit)
    println(same_word)
    println(different)
}
~~~

## Boolean operators

`&&`, `||`, and unary `!` require `Bool`. `&&` and `||` short-circuit: the right operand is evaluated only when needed.

~~~princi
fn is_positive(value: Int) -> Bool {
    return value > 0
}

fn main() {
    let accepted = is_positive(4) && !false
    let fallback = false || accepted
    println(fallback)
}
~~~

Unary `+` and `-` accept `Int` or `Float` values.

## Assignment operators

Assignments are statements. `=` replaces the target value; `+=`, `-=`, and `*=` update mutable targets. `+=` follows the same type rules as `+`, so it also concatenates strings. No `/=` or `%=` operator is defined.

~~~princi
fn main() {
    var count = 5
    count += 2
    count *= 3
    count -= 1
    println(count)
}
~~~

Targets must be mutable. Collection element mutation follows the rules on [Lists](/docs/language/lists).

## Precedence

From lowest to highest: `..`, `||`, `&&`, equality, comparisons, addition/subtraction, multiplication/division/remainder, unary operators, and calls/member access/indexing. Binary operators associate left-to-right. Ranges cannot be chained without parentheses.

~~~princi
fn main() {
    let result = 2 + 3 * 4
    let choice = false || true && false
    println(result) // 14
    println(choice) // false
}
~~~

## Current v0.1 limitations

There is no operator overloading, implicit numeric promotion, exponentiation operator, bitwise operator, or compound division/remainder assignment. Equality is not defined for class or struct values.

## Related topics

- [Primitive types](/docs/language/primitive-types)
- [Strings](/docs/language/strings)
- [Control flow](/docs/language/control-flow)
- [Syntax and precedence](/docs/language/syntax)
