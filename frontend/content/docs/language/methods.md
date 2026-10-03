Methods are functions declared inside classes. They receive the instance as `self`, and can read or update its fields according to the normal type and mutability checks.

~~~princi
class Counter {
    value: Int

    init(value: Int) {
        self.value = value
    }

    fn increment() {
        self.value += 1
    }

    fn current() -> Int {
        return self.value
    }
}

fn main() {
    var counter = Counter(40)
    counter.increment()
    println(counter.current())
}
~~~

The receiver is written before the dot, and method arguments follow in parentheses. The compiler selects the implementation statically from the receiver's declared class type; generated method symbols are qualified by class name and receive a hidden `self` pointer.

## Field access through self

Inside a method, write `self.field` to name a field explicitly. Outside, use `value.field`. Method calls and fields are type-checked against the declared class:

~~~princi
class User {
    name: String

    init(name: String) {
        self.name = name
    }

    fn greeting(prefix: String) -> String {
        return prefix + self.name
    }
}

fn main() {
    let user = User("Lin")
    println(user.greeting("Hello, "))
}
~~~

Method parameters are immutable, and a missing return annotation means `Void`. Methods may call other methods and ordinary functions using normal argument and return checks.

## Visibility and dispatch

Visibility modifiers do not exist in v0.1; class methods and fields are public/module-visible under the current temporary rules. Dispatch is static. A class reference does not enable overriding or virtual dispatch.

## Current v0.1 limitations

Methods exist only on classes. Struct methods are rejected. Method overloading, inheritance, virtual methods, interfaces, traits, abstract methods, static methods, operator overloads, and reflection are unsupported.

## Related topics

- [Classes](/docs/language/classes)
- [Constructors](/docs/language/constructors)
- [Functions](/docs/language/functions)
- [Structs](/docs/language/structs)
