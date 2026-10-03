The lexer and parser are the first two language front-end stages. They transform UTF-8 source text into a located syntax tree while keeping enough source information for precise diagnostics.

## Source locations

`source::SourceFile` loads the file and indexes line starts. Every token and AST node carries a span whose start byte is included and whose end byte is excluded. Diagnostics report one-based line and character column positions and can underline the relevant source excerpt. UTF-8 byte offsets are retained internally, so non-ASCII text does not corrupt span boundaries.

## Lexer

`lexer::lex` scans source text into located `Token` values. Token kinds include identifiers, integer and floating-point literal spellings, decoded string literals, booleans, keywords, operators, punctuation, and end-of-file. Whitespace is skipped; `//` comments continue to the end of a line.

The lexer recognizes the v0.1 keywords and operator spellings documented in [lexical syntax](/docs/reference/lexical-syntax). String literals support a small escape set. Invalid escapes, unterminated strings, malformed exponents, NUL strings, and unexpected characters return source-positioned lexical diagnostics rather than panicking.

~~~text
fn main() {
    let total = 20 + 22 // line comment
    println(total)
}
~~~

The lexer returns an error for malformed input instead of passing an invalid token onward. Unlike parser recovery, lexical failure currently stops token production at the first bad lexical item.

## Parser

`parser::parse` consumes tokens and returns a `ParseResult` containing a `Program` and any syntax diagnostics. It recognizes top-level functions, classes, structs, imports, and `extern "C"` declarations, plus the v0.1 statements and expressions.

Expressions use a Pratt parser. Binary operators follow the documented precedence; equal-precedence binary operators group left to right, while range expressions cannot chain unless explicitly parenthesized. Calls, member access, and indexing bind most tightly.

~~~princi
fn main() {
    let value = 2 + 3 * 4
    println(value)
}
~~~

The parser synchronizes after errors at declaration, member, and statement boundaries. It can therefore report multiple syntax errors from a file without consuming an enclosing closing brace accidentally. Semantic correctness is a later stage; parsing alone does not prove that identifiers or types exist.

## Related topics

- [AST](/docs/compiler/ast)
- [Syntax guide](/docs/language/syntax)
- [Diagnostics](/docs/compiler/diagnostics)
- [Compilation pipeline](/docs/compiler/compilation-pipeline)
