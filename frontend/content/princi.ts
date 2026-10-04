export const factorialSource = `fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }

    return n * factorial(n - 1)
}

fn main() {
    let result = factorial(5)
    println(result)
}`;

export const variablesSource = `fn main() {
    let pi: Float = 3.14159
    var count = 2
    count += 1
    println(count)
}`;

export const functionSource = `fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() {
    println(add(20, 22))
}`;

export const controlFlowSource = `fn main() {
    var count = 0

    if count == 0 {
        println("starting")
    } else {
        println("already running")
    }

    while count < 2 {
        count += 1
    }

    for index in 0..3 {
        println(index)
    }
}`;

export const listSource = `fn main() {
    var numbers: List<Int> = [1, 2, 3]
    numbers[1] = 7
    numbers.add(9)

    println(numbers.length)
    println(numbers[1])
}`;

export const classSource = `class User {
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
}`;

export const structCopySource = `struct Point {
    x: Float
    y: Float
}

fn main() {
    var point = Point(3.0, 4.0)
    let snapshot = point
    point.x = 5.0
    println(snapshot.x)
    println(point.x)
}`;

export const nativePipeline = [
  { title: "Source", detail: ".prnc / .princi" },
  { title: "Lexer", detail: "Tokens with source spans" },
  { title: "Parser + AST", detail: "Structured syntax tree" },
  { title: "Module resolution", detail: "Built-in registry" },
  { title: "Semantic analysis", detail: "Typed representation" },
  { title: "LLVM IR", detail: "Verified module" },
  { title: "Windows object", detail: "x86-64 COFF" },
  { title: "Native link", detail: "MinGW-w64 runtime" },
  { title: ".exe", detail: "Windows x86-64" },
];
