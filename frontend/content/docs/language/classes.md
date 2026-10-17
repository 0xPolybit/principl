A class declares a reference-oriented type with typed fields and instance behavior. Classes may be generic and may implement interfaces or traits. The compiler creates an object when a class is constructed; assigning the value copies its reference, so two bindings can refer to the same instance.

~~~princi
class User {
    name: String
    age: Int

    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }

    fn birthday() {
        self.age += 1
    }
}

fn main() {
    var user = User("Alice", 24)
    user.birthday()
    println(user.age)
}
~~~

## Fields and references

Declare fields with a name and explicit type. Fields are stored in declaration order after compiler-managed object metadata. A copied class value retains the same identity:

~~~princi
class Counter {
    value: Int

    init(value: Int) {
        self.value = value
    }
    fn increment() {
        self.value += 1
    }
}

fn main() {
    var first = Counter(1)
    let alias = first
    alias.increment()
    println(first.value) // 2
}
~~~

The reference stored in a `let` binding cannot be replaced. Methods can mutate the referenced instance through `self`; assigning a field directly through a local requires a mutable (`var`) base binding. Collection elements have their own rules on [Lists](/docs/language/lists).

## Visibility and dispatch

There are no visibility modifiers in v0.2. Class fields and methods are public/module-visible under the temporary rules. Calls on a concrete class use static dispatch based on its declared type. Calls through interface or trait values use dynamic dispatch; class inheritance and virtual class methods are not supported.

## Current v0.2 limitations

Classes have no inheritance, virtual class methods, abstract classes, method overloading, operator overloading, or reflection. A class may declare at most one `init`. Generic classes are specialized for concrete type arguments; methods can use enclosing type parameters but cannot declare their own. There is no user-defined destructor or manual allocation/free syntax.

## Related topics

- [Constructors](/docs/language/constructors)
- [Methods](/docs/language/methods)
- [Structs: value types compared](/docs/language/structs)
- [Managed memory](/docs/compiler/managed-memory)
