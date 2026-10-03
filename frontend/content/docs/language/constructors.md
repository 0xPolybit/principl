Class construction initializes a new class instance. A class can declare one `init` constructor. Its parameters have explicit types; the constructor body initializes fields through `self`.

~~~princi
class User {
    name: String
    age: Int

    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }
}

fn main() {
    var user = User("Ada", 36)
    println(user.name)
}
~~~

`Type(arguments)` invokes the constructor and checks its arity and argument types at compile time.

## Construction without init

If a class has no `init`, v0.1 provides positional construction using every field in declaration order:

~~~princi
class Size {
    width: Int
    height: Int
}

fn main() {
    let size = Size(800, 600)
    println(size.width)
}
~~~

Named-field construction can be used for classes and structs. It initializes every declared field exactly once and directly initializes those fields:

~~~princi
class Window {
    title: String
    width: Int
}

fn main() {
    let window = Window { title: "Editor", width: 900 }
    println(window.title)
}
~~~

This form does not call a class `init`; it supplies all fields directly. Use `Type(args)` when constructor logic must run.

## Struct construction

Structs have no `init`; `Point(x, y)` uses field declaration order, or use named-field construction. See [Structs](/docs/language/structs).

## Current v0.1 limitations

Only one class constructor is allowed. There is no constructor overloading, default arguments, named constructor parameters, inheritance, delegating constructors, user-defined allocation, or custom destructor. Constructor visibility follows the same all-public/module-visible temporary rule as other members.

## Related topics

- [Classes](/docs/language/classes)
- [Methods and self](/docs/language/methods)
- [Structs](/docs/language/structs)
- [Variables](/docs/language/variables)
