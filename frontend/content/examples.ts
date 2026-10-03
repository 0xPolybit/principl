export type ExampleDifficulty = "Beginner" | "Intermediate" | "Advanced";

export type PrinciExample = {
  slug: string;
  title: string;
  difficulty: ExampleDifficulty;
  category: "Essentials" | "Control flow" | "Data & objects" | "Interop";
  filename: string;
  concepts: string[];
  source: string;
  explanation: string;
  output?: string;
  docs: { label: string; href: string }[];
};

export const princiExamples: PrinciExample[] = [
  {
    slug: "hello-world",
    title: "Hello World",
    difficulty: "Beginner",
    category: "Essentials",
    filename: "hello.prnc",
    concepts: ["main", "String", "println"],
    source: `fn main() {
    println("Hello from PrinciPL!")
}`,
    explanation:
      "Every executable starts at a zero-argument main function. println writes a String followed by a newline.",
    output: "Hello from PrinciPL!",
    docs: [{ label: "Hello, world", href: "/docs/hello-world" }],
  },
  {
    slug: "variables",
    title: "Variables",
    difficulty: "Beginner",
    category: "Essentials",
    filename: "variables.princi",
    concepts: ["let", "var", "type inference", "compound assignment"],
    source: `fn main() {
    let label: String = "items"
    var count = 2
    count += 1
    println(label)
    println(count)
}`,
    explanation:
      "let binds an immutable value, while var allows reassignment. The compiler infers count as Int from its initializer.",
    output: `items
3`,
    docs: [{ label: "Variables", href: "/docs/language/variables" }],
  },
  {
    slug: "functions",
    title: "Functions",
    difficulty: "Beginner",
    category: "Essentials",
    filename: "functions.prnc",
    concepts: ["parameters", "return types", "function calls"],
    source: `fn add(left: Int, right: Int) -> Int {
    return left + right
}

fn main() {
    println(add(20, 22))
}`,
    explanation:
      "Parameters and return values have declared types. A function without an explicit return type returns Void.",
    output: "42",
    docs: [{ label: "Functions", href: "/docs/language/functions" }],
  },
  {
    slug: "factorial-recursion",
    title: "Factorial / Recursion",
    difficulty: "Intermediate",
    category: "Essentials",
    filename: "factorial.princi",
    concepts: ["recursion", "if", "return", "Int"],
    source: `fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }

    return n * factorial(n - 1)
}

fn main() {
    println(factorial(5))
}`,
    explanation:
      "A function can call itself. This example returns 1 for the base case, then multiplies by the result for the previous integer.",
    output: "120",
    docs: [{ label: "Functions & recursion", href: "/docs/language/functions" }],
  },
  {
    slug: "conditions",
    title: "Conditions",
    difficulty: "Beginner",
    category: "Control flow",
    filename: "conditions.prnc",
    concepts: ["if", "else", "Bool", "comparison"],
    source: `fn main() {
    let score = 72

    if score >= 60 && score < 80 {
        println("pass")
    } else {
        println("keep practicing")
    }
}`,
    explanation:
      "if and else choose a block from a Bool condition. Comparisons produce Bool, and && combines two Boolean expressions.",
    output: "pass",
    docs: [{ label: "Control flow", href: "/docs/language/control-flow" }],
  },
  {
    slug: "while-loop",
    title: "While Loop",
    difficulty: "Beginner",
    category: "Control flow",
    filename: "countdown.princi",
    concepts: ["while", "mutable local", "comparison"],
    source: `fn main() {
    var count = 0

    while count < 3 {
        count += 1
    }

    println(count)
}`,
    explanation:
      "A while loop repeats as long as its Bool condition is true. The loop changes count on each pass.",
    output: "3",
    docs: [{ label: "Loops", href: "/docs/language/loops" }],
  },
  {
    slug: "for-range-loop",
    title: "For Range Loop",
    difficulty: "Beginner",
    category: "Control flow",
    filename: "sum-range.prnc",
    concepts: ["for", "integer range", "end-exclusive bounds"],
    source: `fn main() {
    var total = 0

    for value in 1..5 {
        total += value
    }

    println(total)
}`,
    explanation:
      "The range 1..5 includes 1, 2, 3, and 4. Its end bound is exclusive, so this loop prints their sum.",
    output: "10",
    docs: [{ label: "Loops & ranges", href: "/docs/language/loops" }],
  },
  {
    slug: "strings",
    title: "Strings",
    difficulty: "Beginner",
    category: "Essentials",
    filename: "greeting.princi",
    concepts: ["String", "concatenation", "println"],
    source: `fn main() {
    let greeting = "Hello, " + "PrinciPL!"
    println(greeting)
}`,
    explanation:
      "The + operator concatenates two strings and creates a new String value. String interpolation is not part of v0.1.",
    output: "Hello, PrinciPL!",
    docs: [{ label: "Strings", href: "/docs/language/strings" }],
  },
  {
    slug: "classes",
    title: "Classes",
    difficulty: "Intermediate",
    category: "Data & objects",
    filename: "user.prnc",
    concepts: ["class", "reference value", "field access"],
    source: `class User {
    name: String
    age: Int

    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }
}

fn main() {
    var user = User("Ada", 20)
    println(user.name)
}`,
    explanation:
      "A class groups typed instance fields. Class values are managed references, and fields are accessible under the v0.1 public-by-default rule.",
    output: "Ada",
    docs: [{ label: "Classes", href: "/docs/language/classes" }],
  },
  {
    slug: "constructor",
    title: "Constructor",
    difficulty: "Intermediate",
    category: "Data & objects",
    filename: "message.princi",
    concepts: ["init", "self", "object construction"],
    source: `class Message {
    text: String

    init(text: String) {
        self.text = text
    }
}

fn main() {
    let message = Message("Ready")
    println(message.text)
}`,
    explanation:
      "A class may declare one init constructor. Its parameters are passed to positional construction and can initialize fields through self.",
    output: "Ready",
    docs: [{ label: "Constructors", href: "/docs/language/constructors" }],
  },
  {
    slug: "methods",
    title: "Methods",
    difficulty: "Intermediate",
    category: "Data & objects",
    filename: "counter.prnc",
    concepts: ["instance methods", "self", "static dispatch"],
    source: `class Counter {
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
    var counter = Counter(1)
    counter.increment()
    println(counter.current())
}`,
    explanation:
      "Instance methods use self to access the receiver. v0.1 resolves calls statically from the receiver's class type.",
    output: "2",
    docs: [{ label: "Methods", href: "/docs/language/methods" }],
  },
  {
    slug: "structs",
    title: "Structs",
    difficulty: "Intermediate",
    category: "Data & objects",
    filename: "point.princi",
    concepts: ["struct", "value semantics", "typed fields"],
    source: `struct Point {
    x: Float
    y: Float
}

fn distanceSquared(point: Point) -> Float {
    return point.x * point.x + point.y * point.y
}

fn main() {
    let point = Point(3.0, 4.0)
    println(distanceSquared(point))
}`,
    explanation:
      "Structs store fields inline and copy by value when passed or returned. This function reads a Point without changing it.",
    output: "25",
    docs: [{ label: "Structs", href: "/docs/language/structs" }],
  },
  {
    slug: "lists",
    title: "List<T>",
    difficulty: "Intermediate",
    category: "Data & objects",
    filename: "numbers.prnc",
    concepts: ["List<Int>", "indexing", "add", "length"],
    source: `fn main() {
    var numbers: List<Int> = [1, 2, 3]
    numbers[1] = 7
    numbers.add(9)

    println(numbers.length)
    println(numbers[1])
}`,
    explanation:
      "List<T> checks all values against one element type. Index reads and writes have runtime bounds checks; add appends an item.",
    output: `4
7`,
    docs: [{ label: "Lists", href: "/docs/language/lists" }],
  },
  {
    slug: "imports",
    title: "Imports",
    difficulty: "Beginner",
    category: "Interop",
    filename: "modules.princi",
    concepts: ["import io", "import math", "built-in modules"],
    source: `import io
import math

fn main() {
    println("Built-in imports resolved")
}`,
    explanation:
      "Only the built-in io and math module names resolve in v0.1. Imports do not create namespaces, and math has no exported functions yet.",
    output: "Built-in imports resolved",
    docs: [{ label: "Imports", href: "/docs/language/imports" }],
  },
  {
    slug: "c-ffi",
    title: "Limited C FFI",
    difficulty: "Advanced",
    category: "Interop",
    filename: "abs.prnc",
    concepts: [`extern "C"`, "Int32", "Windows x86-64 ABI"],
    source: `extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    println(abs(-42))
}`,
    explanation:
      "This declares a direct C ABI function using a supported primitive signature. The Windows C runtime supplies abs; managed values and pointers cannot cross this boundary.",
    output: "42",
    docs: [{ label: "C interoperability", href: "/docs/language/c-ffi" }],
  },
];
