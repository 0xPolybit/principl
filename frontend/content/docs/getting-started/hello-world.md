This walkthrough creates a Princi source file, compiles it to a native Windows executable, and runs that executable from PowerShell.

## 1. Create the source file

In a working directory, create `hello.prnc` with this content:

```princi showLineNumbers
fn main() {
    println("Hello from PrinciPL!")
}
```

`main` is the entry point and takes no parameters. `println` writes the string followed by a newline.

The longer `.princi` extension is an exact alias. Save the same program as `hello.princi` if preferred; it uses the same grammar, checks, and compiler pipeline.

## 2. Build the executable

Run the build command in the directory containing the source:

```powershell
princi build hello.prnc
```

The compiler creates `hello.exe` in that directory. With the aliased extension, run `princi build hello.princi`; it also creates `hello.exe`.

## 3. Run it

In PowerShell, launch the generated program with:

```powershell
.\hello.exe
```

Expected output:

```text
Hello from PrinciPL!
```

Compilation and execution are separate steps. v0.1 has no `princi run` command; build first, then launch the `.exe` with Windows.

## A first real program: recursion and a condition

This complete program computes a factorial, stores the result in a local binding, and selects output with an `if`/`else` condition:

```princi
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }

    return n * factorial(n - 1)
}

fn main() {
    let result = factorial(5)

    if result == 120 {
        println("5 factorial is:")
        println(result)
    } else {
        println("Unexpected result")
    }
}
```

Save it as `factorial.princi` (or `factorial.prnc`), then build and run it:

```powershell
princi build factorial.princi
.\factorial.exe
```

Expected output:

```text
5 factorial is:
120
```

The example uses syntax supported by the v0.1 native backend: typed function parameters and returns, integer comparison and multiplication, recursion, local type inference, conditional branches, and `println` for `String` and `Int` values. Continue to [compiling programs](/docs/compiling) to learn how to choose the output path.
