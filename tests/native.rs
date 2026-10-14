use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(windows)]
static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

#[cfg(windows)]
struct TestDir(PathBuf);

#[cfg(windows)]
impl TestDir {
    fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let target_dir = std::env::current_exe()
            .expect("test executable path should be available")
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("test executable should live under the Cargo target directory")
            .to_path_buf();
        let path = target_dir.join("princi-native-tests").join(format!(
            "princi native integration-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        Self(path)
    }

    fn build(&self, name: &str, source: &str) -> PathBuf {
        let source_path = self.0.join(name);
        let output_path = source_path.with_extension("exe");
        fs::write(&source_path, source).expect("Princi source should be written");

        let build = Command::new(env!("CARGO_BIN_EXE_princi"))
            .arg("build")
            .arg(&source_path)
            .arg("-o")
            .arg(&output_path)
            .env("TMP", &self.0)
            .env("TEMP", &self.0)
            .output()
            .expect("compiler should start");
        assert!(
            build.status.success(),
            "build failed: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(output_path.is_file(), "compiler should write an executable");
        output_path
    }

    fn build_and_run(&self, name: &str, source: &str) -> String {
        let output_path = self.build(name, source);
        let run = Command::new(output_path)
            .output()
            .expect("generated Windows executable should run");
        assert!(
            run.status.success(),
            "generated program failed: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n")
    }
}

#[cfg(windows)]
fn native_toolchain_available() -> bool {
    let clang = std::env::var_os("PRINCI_CLANG").unwrap_or_else(|| "clang".into());
    let linker = std::env::var_os("PRINCI_CC").unwrap_or_else(|| "gcc".into());
    Command::new(clang)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
        && Command::new(linker)
            .arg("-dumpmachine")
            .output()
            .is_ok_and(|output| {
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .starts_with("x86_64-w64-mingw32")
            })
}

#[cfg(windows)]
fn skip_without_native_toolchain() -> bool {
    if native_toolchain_available() {
        false
    } else {
        let reason = "install LLVM/Clang with IR and X86 support and x86-64 MinGW-w64 GCC";
        if std::env::var_os("PRINCI_REQUIRE_NATIVE_TESTS").is_some() {
            panic!("{reason}; PRINCI_REQUIRE_NATIVE_TESTS is set");
        }
        eprintln!("skipping Windows executable integration check: {reason}");
        true
    }
}

#[cfg(windows)]
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(windows)]
#[test]
fn factorial_recursion_mutation_branches_while_and_integer_ranges_execute() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "procedural.prnc",
        r#"fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

fn sum_to(end: Int) -> Int {
    var total = 0
    for index in 0..end {
        total += index
    }
    return total
}

fn main() {
    var result = factorial(5)
    var count = 0
    while count < 3 {
        result += 1
        count += 1
    }
    if result == 123 {
        let result = 7
        println(result)
    } else {
        let result = 0
        println(result)
    }
    println(result)
    println(sum_to(4))
}"#,
    );
    assert_eq!(stdout, "7\n123\n6\n");
}

#[cfg(windows)]
#[test]
fn floats_booleans_strings_immutable_bindings_and_functions_execute() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "values.princi",
        r#"fn area(width: Float, height: Float) -> Float {
    return width * height
}

fn greet(name: String) -> String {
    return "Hello, " + name
}

fn main() {
    let measurement = area(2.0, 1.0)
    let ready: Bool = 1.5 < measurement && true
    let greeting = greet("Mira")
    if ready {
        println(greeting)
    } else {
        println("not ready")
    }
    println(measurement)
    println(ready)
    println(!ready || false)
    println(measurement > 1.0 && measurement != 0.0)
    var count: Int = 1
    count *= 4
    println(count)
    println(9223372036854775807 + 1)
    println(-9223372036854775808)
}"#,
    );
    assert_eq!(
        stdout,
        "Hello, Mira\n2\ntrue\nfalse\ntrue\n4\n-9223372036854775808\n-9223372036854775808\n"
    );
}

#[cfg(windows)]
#[test]
fn simple_and_generic_enum_values_move_through_functions_and_execute() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"struct Point { x: Int }
struct Cell<T> { value: T }
class Box<T> {
    value: T
    init(value: T) { self.value = value }
}
class User {
    name: String
    init(name: String) { self.name = name }
}
enum Direction { North South East West }
enum NetworkState { Connected Connecting Disconnected Failed(String) }
enum Maybe<T> { Some(T) None }
enum Result<T, E> { Ok(T) Err(E) }
enum Envelope<T> { Number(Int) Measure(Float) Flag(Bool) Point(Point) User(User) Text(String) Values(List<T>) Nested(Maybe<T>) GenericClass(Box<T>) GenericStruct(Cell<T>) }
fn forward<U>(value: U) -> U {
    return value
}
fn main() {
    let direction = Direction.North
    let state = NetworkState.Failed("Connection refused")
    let maybe = Maybe<Result<Int, String>>.Some(forward(Result<Int, String>.Ok(42)))
    let absent = Maybe<Int>.None
    let point = forward(Envelope<Int>.Point(Point(3)))
    let measure = forward(Envelope<Int>.Measure(2.5))
    let flag = forward(Envelope<Int>.Flag(true))
    let user = forward(Envelope<Int>.User(User("Mira")))
    let text = forward(Envelope<Int>.Text("payload"))
    let values = forward(Envelope<Int>.Values([1, 2, 3]))
    let nested = forward(Envelope<Int>.Nested(Maybe<Int>.Some(9)))
    let generic_class = forward(Envelope<Int>.GenericClass(Box(11)))
    let generic_struct = forward(Envelope<Int>.GenericStruct(Cell<Int>(12)))
    var numbers: List<Envelope<Int>> = [Envelope<Int>.Number(7)]
    numbers.add(Envelope<Int>.Number(8))
    let moved = forward(numbers[1])
    println(numbers.length)
    println("enum aggregate round-trip succeeded")
}"#;
    for file_name in ["enums.prnc", "enums.princi"] {
        let stdout = dir.build_and_run(file_name, source);
        assert_eq!(stdout, "2\nenum aggregate round-trip succeeded\n");
    }
}

#[cfg(windows)]
#[test]
fn integer_main_return_becomes_the_process_exit_status() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let executable = dir.build("exit.prnc", "fn main() -> Int { return 17 }");
    let run = Command::new(executable)
        .output()
        .expect("generated Windows executable should run");

    assert_eq!(run.status.code(), Some(17));
}

#[cfg(windows)]
#[test]
fn short_circuit_boolean_operators_skip_the_unneeded_call() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "short circuit.princi",
        r#"fn side_effect() -> Bool {
    println(99)
    return true
}

fn main() {
    let and_result = false && side_effect()
    let or_result = true || side_effect()
    println(and_result)
    println(or_result)
}"#,
    );
    assert_eq!(stdout, "false\ntrue\n");
}

#[cfg(windows)]
#[test]
fn llvm_string_constants_preserve_escapes_and_utf8() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "string escapes.prnc",
        r#"fn main() {
    println("quoted: \"file\\name\"\n雪")
}"#,
    );
    assert_eq!(stdout, "quoted: \"file\\name\"\n雪\n");
}

#[cfg(windows)]
#[test]
fn print_and_println_have_distinct_newline_behavior() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "print behavior.prnc",
        r#"fn main() {
    print("value:")
    print(42)
    println("!")
    println(7)
}"#,
    );
    assert_eq!(stdout, "value:42!\n7\n");
}

#[cfg(windows)]
#[test]
fn classes_initialize_mutate_fields_and_dispatch_methods() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "classes.prnc",
        r#"class User {
    name: String
    age: Int

    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }

    fn birthday() {
        self.age += 1
    }

    fn getAge() -> Int {
        return self.age
    }

    fn greeting() -> String {
        return "Hello, " + self.name
    }
}

fn main() {
    var user = User("Alice", 24)
    user.birthday()
    println(user.greeting())
    println(user.getAge())
}"#,
    );
    assert_eq!(stdout, "Hello, Alice\n25\n");
}

#[cfg(windows)]
#[test]
fn interfaces_dispatch_by_concrete_class_across_values_fields_lists_and_generic_classes() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"interface Drawable {
    fn draw(prefix: String) -> String
}
interface Clickable {
    fn click() -> Bool
}
class Button implements Drawable, Clickable {
    label: String
    init(label: String) { self.label = label }
    fn draw(prefix: String) -> String { return prefix + self.label }
    fn click() -> Bool { return true }
}
class Circle implements Drawable {
    radius: Int
    init(radius: Int) { self.radius = radius }
    fn draw(prefix: String) -> String { return prefix + "circle" }
}
class Box<T> implements Drawable {
    value: T
    init(value: T) { self.value = value }
    fn draw(prefix: String) -> String { return prefix + "box" }
}
class Holder {
    item: Drawable
    init(item: Drawable) { self.item = item }
    fn get() -> Drawable { return self.item }
}
fn pass(value: Drawable) -> Drawable { return value }
fn render(value: Drawable) { println(value.draw("shape:")) }
fn main() {
    let button = Button("button")
    let circle = Circle(3)
    let box = Box<Int>(4)
    let values: List<Drawable> = [button, circle, box]
    values.add(pass(button))
    let holder = Holder(circle)
    let fromField = holder.get()
    render(fromField)
    println(values[0].draw("draw:"))
    println(values[1].draw("draw:"))
    println(values[2].draw("draw:"))
    values[0] = holder.get()
    println(values[0].draw("updated:"))
    let clickable: Clickable = button
    println(clickable.click())
}"#;

    let prnc_stdout = dir.build_and_run("interfaces.prnc", source);
    let princi_stdout = dir.build_and_run("interfaces.princi", source);
    let expected = "shape:circle\ndraw:button\ndraw:circle\ndraw:box\nupdated:circle\ntrue\n";
    assert_eq!(prnc_stdout, expected);
    assert_eq!(princi_stdout, expected);
}

#[cfg(windows)]
#[test]
fn traits_dispatch_defaults_overrides_multiple_traits_and_box_struct_copies() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"trait Named {
    fn name() -> String
    fn describe() { println("Hello, " + self.name()) }
}
trait Tagged {
    fn tag() -> String
    fn showTag() { println(self.tag()) }
}
class User implements Named, Tagged {
    value: String
    init(value: String) { self.value = value }
    fn name() -> String { return self.value }
    fn tag() -> String { return "member" }
    fn describe() { println("User: " + self.name()) }
}
class GenericUser<T> implements Named {
    value: T
    init(value: T) { self.value = value }
    fn name() -> String { return "generic" }
}
struct Point implements Named {
    value: String
    fn name() -> String { return self.value }
}
struct GenericPoint<T> implements Named {
    value: T
    fn name() -> String { return "generic point" }
}
fn show(value: Named) { value.describe() }
fn pass(value: Named) -> Named { return value }
class Holder {
    value: Named
    init(value: Named) { self.value = value }
    fn get() -> Named { return self.value }
}
fn main() {
    var user = User("Ada")
    let named: Named = user
    user.value = "Grace"
    show(pass(named))
    let tagged: Tagged = user
    tagged.showTag()
    show(GenericUser<Int>(1))

    var point = Point("origin")
    let pointView: Named = point
    point.value = "moved"
    show(pointView)
    let holder = Holder(user)
    show(holder.get())
    let values: List<Named> = [user, point]
    values.add(pointView)
    show(values[1])
    show(values[2])
    show(GenericPoint<Int>(3))
}"#;

    let prnc_stdout = dir.build_and_run("traits.prnc", source);
    let princi_stdout = dir.build_and_run("traits.princi", source);
    let expected = "User: Grace\nmember\nHello, generic\nHello, origin\nUser: Grace\nHello, moved\nHello, origin\nHello, generic point\n";
    assert_eq!(prnc_stdout, expected);
    assert_eq!(princi_stdout, expected);
}

#[cfg(windows)]
#[test]
fn classes_support_default_and_named_field_construction() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "class construction.princi",
        r#"class Counter {
    value: Int

    fn increment() {
        self.value += 1
    }

    fn get() -> Int {
        return self.value
    }
}

class Meter {
    value: Int

    fn get() -> Int {
        return self.value * 10
    }
}

fn main() {
    var positional = Counter(2)
    var named = Counter { value: 4 }
    var meter = Meter(3)
    positional.increment()
    println(positional.get())
    println(named.get())
    println(meter.get())
}"#,
    );
    assert_eq!(stdout, "3\n4\n30\n");
}

#[cfg(windows)]
#[test]
fn structs_copy_by_value_while_class_values_keep_reference_identity() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "struct value semantics.princi",
        r#"struct Segment {
    start: Point
    end: Point
}

struct Point {
    x: Float
    y: Float
}

struct CounterSlot {
    counter: Counter
}

fn moveRight(point: Point) -> Point {
    var moved = point
    moved.x += 1.0
    return moved
}

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
    var original = Point(3.0, 4.0)
    let snapshot = original
    var moved = moveRight(snapshot)
    moved.x += 2.0
    println(original.x)
    println(snapshot.x)
    println(moved.x)

    var segment = Segment(Point(1.0, 2.0), Point(3.0, 4.0))
    segment.start.x = 9.0
    println(segment.start.x)

    let named = Point { x: 7.0, y: 8.0 }
    println(named.y)

    var counter = Counter(1)
    let alias = counter
    alias.increment()
    println(counter.current())

    let slot = CounterSlot(counter)
    let slotCopy = slot
    slotCopy.counter.increment()
    println(counter.current())
}"#,
    );
    assert_eq!(stdout, "3\n3\n6\n9\n8\n2\n3\n");
}

#[cfg(windows)]
#[test]
fn typed_lists_infer_construct_index_mutate_append_and_pass_to_functions() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "list operations.princi",
        r#"fn summarize(values: List<Int>) -> Int {
    values[0] += 2
    values.add(10)
    return values[0] + values.length
}

fn make_values() -> List<Int> {
    return [4, 5]
}

fn main() {
    var numbers: List<Int> = [1, 2, 3]
    numbers[1] = 7
    let inferred = make_values()
    let alias = inferred
    alias.add(6)
    println(numbers.length)
    println(numbers[1])
    println(summarize(numbers))
    println(inferred[0])
    println(inferred.length)
    println(numbers.length)
}"#,
    );
    assert_eq!(stdout, "3\n7\n7\n4\n3\n4\n");
}

#[cfg(windows)]
#[test]
fn nested_lists_and_string_elements_execute() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "nested lists.prnc",
        r#"fn main() {
    var rows: List<List<Int>> = [[1, 2], [3]]
    rows[0].add(4)
    println(rows.length)
    println(rows[0].length)
    println(rows[0][2])

    var names: List<String> = ["Ada", "Lin"]
    names[1] = names[0] + " Lovelace"
    names.add("Grace")
    println(names[1])
    println(names.length)
}"#,
    );
    assert_eq!(stdout, "2\n3\n4\nAda Lovelace\n3\n");
}

#[cfg(windows)]
#[test]
fn lists_store_generic_class_references_and_generic_struct_values() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"class Box<T> {
    value: T

    init(value: T) {
        self.value = value
    }

    fn get() -> T {
        return self.value
    }
}

struct Pair<A, B> {
    first: A
    second: B
}

fn main() {
    var boxes: List<Box<Int>> = [Box<Int>(10)]
    boxes.add(Box<Int>(11))
    boxes[0] = Box<Int>(12)
    var shared = boxes[0]
    shared.value = 13
    boxes[1].value = 14

    var emptyBoxes: List<Box<Int>> = []
    emptyBoxes.add(Box<Int>(19))
    let inferredBoxes = [Box<Int>(20)]
    let inferredPairs = [Pair<String, Float>("inferred", 3.5)]

    var pairs: List<Pair<String, Float>> = [Pair<String, Float>("one", 1.5)]
    pairs.add(Pair<String, Float>("two", 2.5))
    pairs[0] = Pair<String, Float>("zero", 0.5)
    var copy = pairs[1]
    copy.first = "copy"
    println(pairs[1].first)
    pairs[1] = copy

    println(boxes.length)
    println(boxes[0].get())
    println(boxes[1].get())
    println(emptyBoxes[0].get())
    println(inferredBoxes[0].get())
    println(pairs.length)
    println(pairs[0].first)
    println(pairs[0].second)
    println(pairs[1].first)
    println(pairs[1].second)
    println(inferredPairs[0].first)
    println(inferredPairs[0].second)
}"#;

    let prnc_stdout = dir.build_and_run("generic lists.prnc", source);
    let princi_stdout = dir.build_and_run("generic lists.princi", source);
    let expected = "two\n2\n13\n14\n19\n20\n2\nzero\n0.5\ncopy\n2.5\ninferred\n3.5\n";
    assert_eq!(prnc_stdout, expected);
    assert_eq!(princi_stdout, expected);
}

#[cfg(windows)]
#[test]
fn list_indexing_reports_negative_and_past_end_bounds_errors() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    for (name, index) in [
        ("negative list index.prnc", "-1"),
        ("past end list index.prnc", "1"),
    ] {
        let executable = dir.build(
            name,
            &format!("fn main() {{\n    let values = [42]\n    println(values[{index}])\n}}"),
        );
        let run = Command::new(executable)
            .output()
            .expect("generated Windows executable should run");
        assert_eq!(run.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&run.stdout).contains("list index out of bounds"),
            "unexpected bounds diagnostic: {}",
            String::from_utf8_lossy(&run.stdout)
        );
    }
}

#[cfg(windows)]
#[test]
fn process_lifetime_managed_heap_survives_temporary_cycles_and_growth_stress() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "managed memory stress.prnc",
        r#"class StressNode {
    links: List<StressNode>
    value: Int

    init(links: List<StressNode>, value: Int) {
        self.links = links
        self.value = value
    }
}

fn main() {
    let retained: List<StressNode> = []
    let retained_strings: List<String> = []
    var index = 0
    while index < 10000 {
        let outgoing: List<StressNode> = []
        let node = StressNode(outgoing, index)
        outgoing.add(node)

        let phrase = "managed" + " memory"
        let decorated = phrase + "!"
        let temporary_strings = [phrase, decorated]
        if index % 1000 == 0 {
            retained.add(node)
            retained_strings.add(temporary_strings[1])
        }
        index += 1
    }

    println(retained.length)
    println(retained[9].value)
    println(retained[9].links[0].value)
    println(retained[9].links.length)
    println(retained_strings.length)
    println(retained_strings[9])
}"#,
    );
    assert_eq!(stdout, "10\n9000\n9000\n1\n10\nmanaged memory!\n");
}

#[cfg(windows)]
#[test]
fn calls_windows_c_runtime_function_through_extern_c() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let stdout = dir.build_and_run(
        "c ffi abs.prnc",
        r#"extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    println(abs(-42))
}"#,
    );
    assert_eq!(stdout, "42\n");
}

#[cfg(windows)]
#[test]
fn generic_functions_monomorphize_and_run_for_both_source_extensions() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"fn identity<T>(value: T) -> T {
    return value
}

fn first<A, B>(a: A, b: B) -> A {
    return a
}

fn repeat<T>(remaining: Int, value: T) -> T {
    if remaining == 0 {
        return value
    }
    return repeat<T>(remaining - 1, value)
}

struct Point {
    x: Int
}

class User {
    name: String

    init(name: String) {
        self.name = name
    }
}

fn main() {
    let number = identity<Int>(42)
    let inferred = identity(7)
    let nested = identity(identity(5))
    let greeting = identity<String>("hello")
    let values: List<Int> = identity<List<Int>>([1, 2, 3])
    let chosen = first<Int, String>(number, greeting)
    let point = identity(Point(9))
    let user = identity<User>(User("Ada"))

    println(number)
    println(inferred)
    println(nested)
    println(greeting)
    println(values.length)
    println(chosen)
    println(repeat(3, "recursive"))
    println(point.x)
    println(user.name)
}"#;

    let prnc_stdout = dir.build_and_run("generic functions.prnc", source);
    let princi_stdout = dir.build_and_run("generic functions.princi", source);
    let expected = "42\n7\n5\nhello\n3\n42\nrecursive\n9\nAda\n";
    assert_eq!(prnc_stdout, expected);
    assert_eq!(princi_stdout, expected);
}

#[cfg(windows)]
#[test]
fn bounded_generics_dispatch_traits_and_interfaces_for_classes_and_structs() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"trait Named { fn name() -> String }
trait Described { fn description() -> String { return "default" } }
trait Marked { fn marker() -> String { return "marked" } }
interface Drawable { fn draw() -> String }

class User implements Named, Described, Marked, Drawable {
    value: String
    init(value: String) { self.value = value }
    fn name() -> String { return self.value }
    fn description() -> String { return "user:" + self.name() }
    fn draw() -> String { return "draw:" + self.name() }
}

class Bot implements Named, Described, Marked, Drawable {
    value: String
    init(value: String) { self.value = value }
    fn name() -> String { return self.value }
    fn draw() -> String { return "draw:" + self.name() }
}

struct Point implements Named {
    value: String
    fn name() -> String { return self.value }
}

fn describe<T: Named + Described + Marked>(value: T) -> String {
    return value.name() + "|" + value.description() + "|" + value.marker()
}

fn paint<T: Drawable>(value: T) {
    println(value.draw())
}

class Repository<T: Named> {
    value: T
    init(value: T) { self.value = value }
    fn name() -> String { return self.value.name() }
}

fn repositoryName<T: Named>(repository: Repository<T>) -> String {
    return repository.name()
}

struct Wrapper<T: Named> implements Named {
    value: T
    fn name() -> String { return self.value.name() }
}

fn main() {
    let user = User("Grace")
    let bot = Bot("Hal")
    println(describe(user))
    println(describe<User>(user))
    println(describe(bot))
    paint(user)

    let repository = Repository<User>(user)
    println(repository.name())
    println(repositoryName(repository))

    let point = Point { value: "origin" }
    let wrapper = Wrapper<Point> { value: point }
    println(wrapper.name())
}"#;

    let prnc_stdout = dir.build_and_run("bounded generics.prnc", source);
    let princi_stdout = dir.build_and_run("bounded generics.princi", source);
    let expected = "Grace|user:Grace|marked\nGrace|user:Grace|marked\nHal|default|marked\ndraw:Grace\nGrace\nGrace\norigin\n";
    assert_eq!(prnc_stdout, expected);
    assert_eq!(princi_stdout, expected);
}

#[cfg(windows)]
#[test]
fn generic_classes_and_structs_specialize_and_run_for_both_extensions() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let source = r#"class Box<T> {
    value: T

    init(value: T) {
        self.value = value
    }

    fn get() -> T {
        return self.value
    }

    fn replace(next: T) {
        self.value = next
    }
}

struct Pair<A, B> {
    first: A
    second: B
}

fn retain<T>(value: Box<T>) -> Box<T> {
    return value
}

fn keepPair<A, B>(value: Pair<A, B>) -> Pair<A, B> {
    return value
}

fn main() {
    let numberBox = Box<Int>(42)
    let textBox = Box<String>("hello")
    let inferred = Box(7)
    let nested = Box<List<Int>>([3, 4])
    let boxes: List<Box<String>> = [textBox]
    let nestedPair = Pair<Box<Int>, List<Float>>(numberBox, [1.5])
    let namedPair: Pair<String, Float> = Pair<String, Float> {
        first: "named",
        second: 2.5
    }

    numberBox.replace(43)
    var pair = Pair<Int, String>(1, "one")
    let copied = pair
    pair.first = 2
    let copiedPair = keepPair(pair)
    let retainedBox = retain(numberBox)

    println(numberBox.get())
    println(retainedBox.get())
    println(textBox.get())
    println(inferred.get())
    println(pair.first)
    println(copied.first)
    println(copiedPair.first)
    println(pair.second)
    println(nested.get()[1])
    println(boxes[0].get())
    println(nestedPair.first.get())
    println(namedPair.first)
}"#;

    let prnc = dir.build_and_run("generic types.prnc", source);
    let princi = dir.build_and_run("generic types.princi", source);
    let expected = "43\n43\nhello\n7\n2\n1\n2\none\n4\nhello\n43\nnamed\n";
    assert_eq!(prnc, expected);
    assert_eq!(princi, expected);
}

#[cfg(windows)]
#[test]
fn rejects_int32_out_of_range_before_crossing_the_c_abi() {
    if skip_without_native_toolchain() {
        return;
    }
    let dir = TestDir::new();
    let executable = dir.build(
        "c ffi int32 range.prnc",
        r#"extern "C" {
    fn abs(value: Int32) -> Int32
}

fn main() {
    print(abs(2147483648))
}"#,
    );
    let run = Command::new(executable)
        .output()
        .expect("generated Windows executable should run");
    assert_eq!(run.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&run.stdout).contains("FFI Int32 argument out of range"));
}
