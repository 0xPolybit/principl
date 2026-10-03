A class declares a reference-oriented type with typed fields and instance behavior. The compiler creates an object when a class is constructed; assigning the value copies its reference, so two bindings can refer to the same instance.

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

There are no visibility modifiers in v0.1. Class fields and methods are accessible under the temporary public/module-visible rules. Calls use static dispatch based on the receiver's declared class type. There are no virtual tables or runtime method overrides.

## Current v0.1 limitations

Classes have no inheritance, virtual methods, interfaces, traits, abstract classes, method overloading, operator overloading, or reflection. A class may declare at most one `init`. There is no user-defined destructor or manual allocation/free syntax.

## Related topics

- [Constructors](/docs/language/constructors)
- [Methods](/docs/language/methods)
- [Structs: value types compared](/docs/language/structs)
- [Managed memory](/docs/compiler/managed-memory)
