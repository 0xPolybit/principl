Classes are reference-oriented types with typed fields, one optional `init` constructor, and instance methods:

~~~princi
class User {
    name: String

    init(name: String) {
        self.name = name
    }

    fn greet() {
        println(self.name)
    }
}

fn main() {
    var user = User("Alice")
    user.greet()
}
~~~

The compiler allocates each class instance on the managed heap. Assigning a class value copies the reference, so aliases refer to the same object. Methods use static dispatch selected from the receiver's declared class.

## Construction and visibility

`Type(args)` calls the declared initializer. Without an initializer, the compiler provides positional construction in field declaration order. Named-field construction also initializes every field directly.

Visibility modifiers are absent in v0.1; class fields and methods are public within the source unit. There is no inheritance, virtual dispatch, abstract class, method overloading, or reflection.
