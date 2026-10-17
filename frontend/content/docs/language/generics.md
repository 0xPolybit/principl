PrinciPL 0.2 supports generic functions, classes, structs, and enums. The compiler specializes a declaration for concrete type arguments at build time, so generic calls use statically checked native code.

## Generic functions

Declare type parameters between the function name and parameter list. Calls may specify type arguments, or infer them from the actual argument types when every parameter can be determined:

~~~princi
fn identity<T>(value: T) -> T {
    return value
}

fn main() {
    let number = identity<Int>(42)
    let text = identity("hello") // infers T = String
    println(number)
    println(text)
}
~~~

Inference uses call arguments only; it does not use the expected result type. If a type parameter cannot be resolved from arguments, provide it explicitly. A generic function compiles only for concrete specializations used by the program. Repeated equivalent calls share one specialization; recursive calls with the same types are supported. Expanding recursion is diagnosed rather than allowed to specialize indefinitely.

## Generic classes and structs

Generic type arguments are explicit at type references and construction sites. Constructor inference is available only when its parameter types determine all type arguments:

~~~princi
class Box<T> {
    value: T

    init(value: T) {
        self.value = value
    }

    fn get() -> T {
        return self.value
    }
}

struct Pair<A, B> {
    first: A
    second: B
}

fn main() {
    let box = Box<Int>(42)
    let pair = Pair<Int, String>(1, "one")
    println(box.get())
    println(pair.second)
}
~~~

The same type syntax nests: `Box<List<Int>>`, `List<Box<String>>`, and `Pair<Box<Int>, List<Float>>`. Class specializations retain reference semantics; struct specializations retain value semantics. Generic enums use explicit concrete type arguments when constructing variants, for example `Maybe<Int>.Some(3)`.

## Bounds

Generic functions, classes, and structs can require declared traits or interfaces. Use `+` for multiple bounds:

~~~princi
trait Named {
    fn name() -> String
}

fn show<T: Named>(value: T) {
    println(value.name())
}
~~~

At specialization, each concrete argument must implement every required contract. A generic parameter can call only methods guaranteed by its bounds. Interface bounds require a conforming class; trait bounds accept conforming classes or structs.

## Current v0.2 limitations

Methods cannot declare their own type parameters; methods on a generic class can use that class's parameters. Traits and interfaces cannot be generic and bounds cannot take type arguments. Generic function inference does not use a result context, and enum variant construction requires explicit generic arguments. There is no runtime generic reflection, specialization syntax, associated type, higher-kinded type, or variadic generic.

## Related topics

- [Classes](/docs/language/classes)
- [Structs](/docs/language/structs)
- [Lists](/docs/language/lists)
- [Traits](/docs/language/traits)
- [Interfaces](/docs/language/interfaces)
- [Enums](/docs/language/enums)
