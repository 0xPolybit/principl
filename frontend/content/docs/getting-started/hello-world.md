Create a file named `hello.prnc`:

~~~princi showLineNumbers {2}
fn main() {
    println("Hello from Princi!")
}
~~~

Build it from PowerShell:

~~~powershell
princi build hello.prnc
~~~

The compiler writes `hello.exe` beside the source. Run it with:

~~~powershell
.\hello.exe
~~~

The program prints:

~~~text
Hello from Princi!
~~~

Use `hello.princi` instead if you prefer the longer suffix. It is fully interchangeable with `.prnc` and does not select another dialect.

## Choose another output name

~~~powershell
princi build hello.prnc -o greeting.exe
~~~

This writes `greeting.exe`. The `-o` option is the only v0.1 build flag.
