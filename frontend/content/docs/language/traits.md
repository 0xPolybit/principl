A trait declares required methods and may provide default method bodies. A class or struct opts into a trait with `implements`, supplies every required method, and may override defaults.

~~~princi
trait Named {
    fn name() -> String

    fn describe() {
        println("Name: " + self.name())
    }
}

class User implements Named {
    value: String

    init(value: String) {
        self.value = value
    }

    fn name() -> String {
        return self.value
    }
}

fn main() {
    let user: Named = User("Ada")
    user.describe()
}
~~~

The `Named` contract requires `name`; `describe` is inherited as a default implementation and invokes the implementation through the trait value. A concrete implementation may override `describe`. Classes and structs can implement multiple traits. A type missing a required method fails conformance checking.

## Default-method conflicts

If two implemented traits provide the same default method and the concrete type does not define that method itself, the compiler reports an ambiguity. Define the method on the concrete type to choose behavior explicitly. There is no implicit trait precedence.

Trait-typed values reuse the runtime dispatch representation used by interface values. They can be used in locals, function parameters and returns, fields, and lists where the concrete value satisfies the trait. Converting a struct to a trait value boxes a copy in managed process-lifetime storage; class values retain their normal reference semantics. Structs do not convert to interface values.

## Current v0.2 limitations

Traits cannot contain fields or constructors and cannot be generic. Methods cannot declare their own type parameters. Trait bounds can constrain generic functions, classes, and structs, but there are no associated types, conditional implementations, negative bounds, or specialization rules.

## Related topics

- [Interfaces](/docs/language/interfaces)
- [Generics and bounds](/docs/language/generics)
- [Structs](/docs/language/structs)
- [Managed memory](/docs/compiler/managed-memory)
