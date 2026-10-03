Compiler diagnostics use a stable code, a plain-language message, and—when the error belongs to a source construct—a file, one-based line and column, source excerpt, and caret span. The diagnostic abstraction is defined in `diagnostics.rs`; lexer, parser, semantic analysis, backend, and CLI return diagnostics instead of printing arbitrary messages.

~~~text
example.prnc:2:20: error[E0201]: expected Int, found String (expression)

    var age: Int = "twenty"
                   ^^^^^^^^
~~~

The span is a half-open UTF-8 byte range internally. `SourceFile` maps it to a character column and a visual underline. Command-line and toolchain errors without a source location use a `princi:` prefix instead.

## Error code groups

| Code | Meaning | Example trigger |
| --- | --- | --- |
| `E0001` | Invalid command-line usage | Unknown command or missing build input |
| `E0002` | Unsupported source extension | Input ends in `.txt` instead of `.prnc` or `.princi` |
| `E0003` | Source path/file error | Source file is missing, unreadable, or not a file |
| `E0100` | Lexical error | Unterminated string or invalid escape |
| `E0101` | Syntax error | `let` has no following binding name |
| `E0201` | Type mismatch | `Int` binding initialized with `String` |
| `E0202` | Unknown identifier/function | Call or name refers to `missing` |
| `E0203` | Unknown type | Parameter names an undeclared type |
| `E0204` | Invalid call | Function call has the wrong argument count |
| `E0205` | Invalid member | Value has no requested field or method |
| `E0206` | Duplicate declaration | Same name is declared twice in one scope |
| `E0207` | Missing or invalid entry point | No valid top-level `main` |
| `E0208` | Invalid return | Missing/wrong value for non-`Void` return |
| `E0209` | Unknown module | Import is not `io` or `math` |
| `E0210` | Uninitialized variable | Local is read before definite initialization |
| `E0299` | Other semantic error | FFI signature contains an unsafe type |
| `E0301` | Unsupported compiler feature | Feature reaches an unsupported backend path |
| `E0401` | LLVM/Clang unavailable | Clang cannot be started |
| `E0403` | Linker unavailable/incompatible | MinGW-w64 GCC missing or wrong target |
| `E0404` | Linker failure | Native executable link fails |
| `E0405` | Build output failure | Temporary/output file cannot be written or moved |
| `E9001` | Unexpected internal compiler failure | Panic is caught at the process boundary |
| `E9002` | LLVM/native generation failure | LLVM IR cannot be verified or emitted as a Windows object |

These identifiers are defined by the current code in `DiagnosticCode`; wording can improve while the code continues to classify the same failure category.

## Examples

An undefined name uses `E0202`:

~~~princi
fn main() {
    print(missing)
}
~~~

A type mismatch uses `E0201`:

~~~princi
fn main() {
    let age: Int = "twenty"
}
~~~

An invalid FFI type is a semantic failure (`E0299`), so it stops before LLVM/linking. A missing linker uses `E0403`; it is reported as a toolchain error rather than a source error. A native link failure uses `E0404`.

## Failure behavior

The CLI returns a nonzero process status for build errors. Normal malformed source does not print a Rust backtrace. The binary catches an unexpected panic at its outer boundary and reports a generic `E9001` internal compiler error; LLVM verification/native emission failures are labeled separately as `E9002`. Error details are kept concise in ordinary output.

## Related topics

- [Lexer & Parser](/docs/compiler/lexer-parser)
- [Semantic analysis](/docs/compiler/semantic-analysis)
- [Compilation pipeline](/docs/compiler/compilation-pipeline)
- [Windows linking](/docs/compiler/windows-toolchain)
