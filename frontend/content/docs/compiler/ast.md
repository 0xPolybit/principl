The AST is the parser's structured representation of a Princi source file. Its data types are defined in `ast.rs`; the parser attaches source spans to declarations, statements, identifiers, expressions, and field initializers.

## Tree shape

At the top level, `Program` contains declarations such as functions, classes, structs, imports, and foreign-function blocks. Function and initializer declarations contain typed parameters and a block. A block contains statements, including variable declarations, assignments, branches, loops, returns, nested blocks, and expression statements.

Expressions form their own tree. Nodes represent identifiers, `self`, literals, list literals, named-field construction, calls, member access, indexes, unary and binary expressions, ranges, and parenthesized groups.

For example, the initializer in this complete program:

~~~princi
fn main() {
    let total = 1
    let price = 2
    let count = 3
    let result = total + price * count
    println(result)
}
~~~

The initializer is represented with `+` at the root, `total` on its left, and
a `*` expression on its right. Each expression node's span points back to its
range in the original source. The full program compiles and prints `7`.

## Spans and diagnostics

Spans are half-open UTF-8 byte ranges (`start` included, `end` excluded). `SourceFile` maps those ranges to one-based line and character-column values when rendering diagnostics. Keeping spans on the source AST lets later stages point to the relevant original expression or declaration.

## From AST to typed representation

The semantic analyzer keeps the parsed AST and records resolved symbols, module identities, and expression/local/parameter types in a `TypedProgram`. It does not replace source nodes with a second public tree. The backend consumes the AST together with those checked type maps.

~~~text
source → located tokens → Program(AST) → TypedProgram(AST + symbols + types)
~~~

## Current v0.1 limitations

The AST models only the current parsed syntax. It has no nodes for enums, `match`, macros, async functions, traits, inheritance, ownership annotations, or raw pointers. Parsed syntax is not itself a guarantee that a feature reaches native code; semantic analysis and backend support define the complete executable boundary.

## Related topics

- [Lexer & Parser](/docs/compiler/lexer-parser)
- [Semantic analysis](/docs/compiler/semantic-analysis)
- [Type system](/docs/compiler/type-system)
- [Compilation pipeline](/docs/compiler/compilation-pipeline)
