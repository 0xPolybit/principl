v0.1 recognizes the fixed built-in module names `io` and `math`:

~~~princi
import io
import math

fn main() {
    println("ready")
}
~~~

Both imports are optional. `io` names the built-in I/O module; `print` and `println` remain available from the global prelude. `math` is registered but currently exports no functions.

Imports do not create namespaces. For example, `io.println(...)` is not supported. Repeated imports are idempotent; unknown, dotted, or nested module paths are errors.

The compiler does not search the filesystem for source modules. User-defined multi-file modules, relative resolution, registries, and remote packages are outside v0.1.
