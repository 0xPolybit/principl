Imports name a small fixed set of compiler-provided modules. v0.1 recognizes the exact, case-sensitive module names `io` and `math`:

~~~princi
import io
import math

fn main() {
    println("ready")
}
~~~

Both imports are optional. At present, an import validates a name against the built-in registry; it does not create a namespace or bind module members. `print` and `println` are global prelude functions, not `io.println` and `io.print`. `math` is registered, but currently exports no functions.

## Resolution behavior

The resolver maps a module name directly to the built-in registry, independent of the source path or `.prnc`/`.princi` extension. Repeating a known import is idempotent. Unknown names and dotted or nested paths are diagnosed:

~~~princi
import graphics // error: not a v0.1 built-in module
~~~

~~~princi
import io.text // error: nested module imports are unsupported
~~~

## Current v0.1 limitations

Princi does not search directories or load other source files for imports. There is no filesystem module resolution, namespace qualification, cyclic user-module loading, package manager, registry, or remote dependency download. Only `io` and `math` resolve; neither currently adds source-level functions, and `math` has no public operations.

## Related topics

- [Built-in functions](/docs/reference/built-in-functions)
- [Compilation pipeline](/docs/compiler/compilation-pipeline)
- [Roadmap](/docs/project/roadmap)
