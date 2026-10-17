An interface declares required instance-method signatures without fields, constructors, or method bodies. Classes implement interfaces explicitly and must provide compatible methods.

~~~princi
interface Drawable {
    fn draw() -> String
}

class Button implements Drawable {
    fn draw() -> String {
        return "button"
    }
}

fn render(value: Drawable) {
    println(value.draw())
}

fn main() {
    render(Button())
}
~~~

An interface value keeps the concrete object's data together with the dispatch table for that interface. Calls through the interface select the method implementation belonging to the object's class. A class can implement multiple interfaces, and interface values can be used in locals, function parameters and returns, class or struct fields, and `List<Interface>` where the field/collection type is supported.

## Conformance

Compilation fails if a class omits a required method or declares a method with a different parameter or return signature. Unknown interfaces and duplicate implementations are errors. Interfaces cannot be instantiated directly. Interface method dispatch is dynamic; ordinary calls through a concrete class retain static dispatch.

## Current v0.2 limitations

Only classes can be converted to interface values. Structs do not implement interfaces in v0.2, though structs can implement traits. Interfaces have no generic parameters, generic methods, default bodies, fields, constructors, inheritance, or virtual class inheritance.

## Related topics

- [Traits](/docs/language/traits)
- [Generic bounds](/docs/language/generics#bounds)
- [Classes](/docs/language/classes)
- [Diagnostics](/docs/compiler/diagnostics)
