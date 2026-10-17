`match` selects one arm based on a value and is an expression. It can also be used as a statement when arms produce `Void`. Each arm has its own lexical scope; pattern-bound names are immutable and do not escape the arm.

~~~princi
enum NetworkState {
    Connected
    Connecting
    Failed(String)
}

fn describe(state: NetworkState) -> String {
    return match state {
        Connected => "online"
        Connecting => "connecting"
        Failed(reason) => reason
    }
}

fn main() {
    println(describe(NetworkState.Failed("offline")))
}
~~~

Supported patterns include nullary enum variants, payload variants with positional destructuring, nested patterns, immutable bindings, `_`, and primitive literal patterns for `Int`, `Float`, `Bool`, and `String`. If a match is used as a value, all arms must have compatible types. A block arm has type `Void`.

## Exhaustiveness and unreachable arms

Enum matches must cover every variant or include a catch-all (`_` or a binding). The diagnostic lists missing qualified cases when possible. Generic enum coverage checks the variant set independent of payload types. Both `true` and `false` cover `Bool`; other primitive literal matches need a catch-all. Duplicate cases and arms shadowed by a prior catch-all are diagnosed.

~~~princi
enum Result<T, E> {
    Ok(T)
    Err(E)
}

fn printResult(result: Result<Int, String>) {
    match result {
        Ok(value) => println(value)
        Err(error) => println(error)
    }
}
~~~

## Current v0.2 limitations

Match guards, list/array patterns, struct destructuring, and range patterns are not supported. For nested payloads the checker is conservative and may require a catch-all unless a payload pattern is irrefutable.

## Related topics

- [Enums](/docs/language/enums)
- [Result and `?`](/docs/language/result)
- [Control flow](/docs/language/control-flow)
- [Diagnostics](/docs/compiler/diagnostics)
