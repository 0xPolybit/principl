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
        let path = std::env::temp_dir().join(format!(
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
        eprintln!(
            "skipping Windows executable integration check: install LLVM/Clang with IR and X86 support and x86-64 MinGW-w64 GCC"
        );
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
