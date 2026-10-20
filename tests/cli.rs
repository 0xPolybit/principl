use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

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
        let path = target_dir.join("princi-cli-tests").join(format!(
            "princi cli integration-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary test directory should be created");
        Self(path)
    }

    fn source(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, "").expect("temporary source should be written");
        path
    }

    fn compiler(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_princi"));
        command.env("TMP", &self.0).env("TEMP", &self.0);
        command
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

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[cfg(windows)]
fn both_extensions_build_native_executables() {
    if !native_toolchain_available() {
        let reason = "install LLVM/Clang with IR and X86 support and x86-64 MinGW-w64 GCC";
        if std::env::var_os("PRINCI_REQUIRE_NATIVE_TESTS").is_some() {
            panic!("{reason}; PRINCI_REQUIRE_NATIVE_TESTS is set");
        }
        eprintln!("skipping Windows executable integration check: {reason}");
        return;
    }
    let dir = TestDir::new();

    for name in ["hello.prnc", "hello.princi"] {
        let source = dir.source(name);
        fs::write(&source, "fn main() { println(1 + 2 * 3) }")
            .expect("valid Princi program should be written");
        let output = source.with_extension("exe");
        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("CLI process should start");

        assert!(
            result.status.success(),
            "native compilation should succeed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            output.is_file(),
            "build should produce {}",
            output.display()
        );
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.contains("unsupported source extension"),
            "{name} should be accepted by extension validation"
        );
        let execution = Command::new(&output)
            .output()
            .expect("generated Windows executable should run");
        assert!(execution.status.success());
        assert_eq!(
            String::from_utf8_lossy(&execution.stdout).replace("\r\n", "\n"),
            "7\n"
        );
    }
}

#[test]
#[cfg(windows)]
fn missing_llvm_compiler_has_an_actionable_diagnostic() {
    let dir = TestDir::new();
    let source = dir.0.join("valid.prnc");
    fs::write(&source, "fn main() {}").expect("valid source should be written");
    let missing_clang = dir.0.join("missing clang.exe");
    let result = dir
        .compiler()
        .arg("build")
        .arg(&source)
        .env("PRINCI_CLANG", missing_clang)
        .output()
        .expect("CLI process should start");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("toolchain error[E0401]"));
    assert!(stderr.contains("install LLVM/Clang"));
}

#[test]
#[cfg(windows)]
fn missing_mingw_linker_has_an_actionable_diagnostic() {
    let clang = std::env::var_os("PRINCI_CLANG").unwrap_or_else(|| "clang".into());
    if !Command::new(clang)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        if std::env::var_os("PRINCI_REQUIRE_NATIVE_TESTS").is_some() {
            panic!("LLVM/Clang is required; PRINCI_REQUIRE_NATIVE_TESTS is set");
        }
        eprintln!("skipping linker diagnostic check because LLVM/Clang is unavailable");
        return;
    }

    let dir = TestDir::new();
    let source = dir.0.join("valid.princi");
    fs::write(&source, "fn main() {}").expect("valid source should be written");
    let missing_linker = dir.0.join("missing MinGW linker.exe");
    let result = dir
        .compiler()
        .arg("build")
        .arg(&source)
        .env("PRINCI_CC", missing_linker)
        .output()
        .expect("CLI process should start");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("toolchain error[E0403]"));
    assert!(stderr.contains("install x86-64 MinGW-w64 GCC"));
}

#[test]
fn unsupported_extensions_and_missing_sources_exit_nonzero() {
    let dir = TestDir::new();
    let unsupported = dir.source("hello.txt");

    for (source, code) in [
        (unsupported, "E0002"),
        (dir.0.join("missing.prnc"), "E0003"),
    ] {
        let result = dir
            .compiler()
            .arg("build")
            .arg(source)
            .output()
            .expect("CLI process should start");
        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains(&format!("error[{code}]")), "{stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
    }
}

#[test]
fn malformed_source_reports_its_file_line_and_column() {
    let dir = TestDir::new();
    for extension in ["prnc", "princi"] {
        let source = dir.0.join(format!("hello.{extension}"));
        fs::write(
            &source,
            "fn main() {\n  let value = 1;\n  let other = 2;\n           \"unfinished\n}\n",
        )
        .expect("malformed source should be written");
        let file_name = source.file_name().unwrap().to_string_lossy();

        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("CLI process should start");

        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains(&format!(
                "{}:4:12: error[E0100]: unterminated string literal",
                file_name
            )),
            "expected a located lexical diagnostic, got: {stderr}"
        );
        assert!(
            stderr.contains("unfinished\n"),
            "source excerpt missing: {stderr}"
        );
        assert!(stderr.contains("^"), "source caret missing: {stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
        assert!(!stderr.contains("stack backtrace"), "{stderr}");
    }
}

#[test]
fn syntax_errors_have_source_excerpt_for_both_extensions() {
    let dir = TestDir::new();
    for extension in ["prnc", "princi"] {
        let source = dir.0.join(format!("syntax.{extension}"));
        fs::write(&source, "fn main() {\n    let = 1\n}\n")
            .expect("invalid source should be written");
        let file_name = source.file_name().unwrap().to_string_lossy();
        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("CLI process should start");

        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains(&format!(
                "{}:2:9: error[E0101]: expected variable name",
                file_name
            )),
            "{stderr}"
        );
        assert!(stderr.contains("    let = 1\n"), "{stderr}");
        assert!(stderr.contains("            ^"), "{stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
    }
}

#[test]
fn semantic_errors_are_reported_before_later_pipeline_stages() {
    let dir = TestDir::new();
    let source = dir.0.join("type_error.princi");
    fs::write(&source, "fn main() {\n    let count: Int = \"wrong\"\n}")
        .expect("source with a type error should be written");
    let file_name = source.file_name().unwrap().to_string_lossy();

    let result = dir
        .compiler()
        .arg("build")
        .arg(&source)
        .output()
        .expect("CLI process should start");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains(&format!(
            "{}:2:22: error[E0201]: expected Int, found String (expression)",
            file_name
        )),
        "expected a located semantic diagnostic, got: {stderr}"
    );
    assert!(
        !stderr.contains("code generation stage"),
        "semantic failures must stop before later pipeline stages"
    );
    assert!(stderr.contains("^"), "source caret missing: {stderr}");
}

#[test]
fn ffi_rejects_managed_values_before_native_linking() {
    let dir = TestDir::new();
    let source = dir.0.join("unsafe_ffi.prnc");
    fs::write(
        &source,
        "extern \"C\" { fn pass_string(value: String) -> String }\nfn main() {}\n",
    )
    .expect("source with an FFI-unsafe signature should be written");

    let result = dir
        .compiler()
        .arg("build")
        .arg(&source)
        .output()
        .expect("CLI process should start");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("error[E0299]: type 'String' is not FFI-safe"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("clang"),
        "semantic FFI errors stop before the backend"
    );
}

#[test]
fn async_syntax_is_identical_but_not_compiled_for_both_extensions_yet() {
    let dir = TestDir::new();
    let source_text = "async fn work() -> Int { return 1 }\nasync fn main() { scope { let task = spawn work()\n let value = await task } }";

    for name in ["async.prnc", "async.princi"] {
        let source = dir.source(name);
        fs::write(&source, source_text).expect("async source should be written");
        let output_path = source.with_extension("exe");
        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("CLI process should start");

        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("error[E0301]"), "{stderr}");
        assert!(
            stderr.contains("async execution is not implemented yet"),
            "{stderr}"
        );
        assert!(
            !output_path.exists(),
            "unsupported async source must not produce {}",
            output_path.display()
        );
    }
}

#[test]
fn common_user_errors_have_stable_codes_and_no_internal_trace() {
    let dir = TestDir::new();
    let cases = [
        (
            "unknown_identifier",
            "fn main() { print(missing) }",
            "E0202",
        ),
        (
            "unknown_type",
            "fn take(value: Missing) {}\nfn main() {}",
            "E0203",
        ),
        (
            "argument_count",
            "fn take(value: Int) {}\nfn main() { take() }",
            "E0204",
        ),
        (
            "invalid_member",
            "fn main() { let value = 1; value.missing }",
            "E0205",
        ),
        (
            "duplicate_declaration",
            "fn main() { let value = 1; let value = 2 }",
            "E0206",
        ),
        ("missing_main", "fn helper() {}", "E0207"),
        ("invalid_return", "fn main() -> Int { return }", "E0208"),
    ];

    for (name, program, code) in cases {
        let source = dir.0.join(format!("{name}.prnc"));
        fs::write(&source, program).expect("test program should be written");
        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("CLI process should start");

        assert!(!result.status.success(), "{name} must fail compilation");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains(&format!("error[{code}]")),
            "{name}: {stderr}"
        );
        assert!(
            stderr.contains("\n\n    "),
            "{name}: source excerpt missing: {stderr}"
        );
        assert!(
            stderr.contains('^'),
            "{name}: source caret missing: {stderr}"
        );
        assert!(!stderr.contains("panicked"), "{name}: {stderr}");
        assert!(!stderr.contains("stack backtrace"), "{name}: {stderr}");
    }
}

#[test]
fn invalid_result_propagation_exits_nonzero_without_emitting_an_executable() {
    let dir = TestDir::new();
    let cases = [
        (
            "wrong-return.prnc",
            "fn read() -> Result<Int, String> { return Result<Int, String>.Ok(1) }\nfn bad() -> Int { return read()? }\nfn main() {}",
            "E0306",
            "the function must return Result<T, E>",
        ),
        (
            "wrong-error.princi",
            "fn read() -> Result<Int, Int> { return Result<Int, Int>.Ok(1) }\nfn bad() -> Result<Int, String> { let value = read()?; return Result<Int, String>.Ok(value) }\nfn main() {}",
            "E0306",
            "cannot propagate error type Int; the enclosing function expects String",
        ),
        (
            "not-result.prnc",
            "fn bad() -> Result<Int, String> { return 1? }\nfn main() {}",
            "E0306",
            "cannot apply '?' to Int; expected Result<T, E>",
        ),
    ];

    for (name, text, code, expected_message) in cases {
        let source = dir.0.join(name);
        fs::write(&source, text).expect("invalid Princi source should be written");
        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("CLI process should start");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "{name} unexpectedly compiled");
        assert!(stderr.contains(&format!("error[{code}]")), "{stderr}");
        assert!(stderr.contains(expected_message), "{stderr}");
        assert!(!source.with_extension("exe").exists());
        assert!(!stderr.contains("panicked"), "{stderr}");
    }
}

#[test]
fn v02_diagnostic_regressions_are_located_actionable_and_hide_backend_details() {
    let dir = TestDir::new();
    let cases = [
        (
            "generic-arity.prnc",
            "fn identity<T>(value: T) -> T { return value }\nfn main() { let item = identity<Int, String>(1) }",
            "E0302",
            "'identity' expects 1 generic argument(s), found 2",
            None,
        ),
        (
            "generic-inference.princi",
            "fn empty<T>() -> List<T> { return [] }\nfn main() { let values = empty() }",
            "E0303",
            "cannot infer generic argument(s) T for 'empty'",
            None,
        ),
        (
            "trait-bound.prnc",
            "trait Printable { fn print() }\nfn show<T: Printable>(value: T) {}\nfn main() { show(1) }",
            "E0213",
            "type Int does not satisfy trait Printable",
            Some("required by fn show<T: Printable>"),
        ),
        (
            "missing-interface.princi",
            "interface Drawable { fn draw(label: String) -> Bool }\nclass Button implements Drawable {}\nfn main() {}",
            "E0211",
            "is missing method 'draw' required by interface 'Drawable'",
            Some("required signature: (String) -> Bool"),
        ),
        (
            "interface-bound-cascade.prnc",
            "interface Drawable { fn draw() }\nclass Button implements Drawable {}\nfn show<T: Drawable>(value: T) {}\nfn main() { show(Button()) }",
            "E0211",
            "is missing method 'draw' required by interface 'Drawable'",
            Some("required signature: () -> Void"),
        ),
        (
            "interface-value-cascade.princi",
            "interface Drawable { fn draw() }\nclass Button implements Drawable {}\nfn render(value: Drawable) {}\nfn main() { render(Button()) }",
            "E0211",
            "is missing method 'draw' required by interface 'Drawable'",
            Some("required signature: () -> Void"),
        ),
        (
            "interface-signature.princi",
            "interface Drawable { fn draw(label: String) -> Bool }\nclass Button implements Drawable { fn draw() {} }\nfn main() {}",
            "E0211",
            "expected (String) -> Bool, found () -> Void",
            None,
        ),
        (
            "trait-default-conflict.prnc",
            "trait First { fn draw() {} }\ntrait Second { fn draw() {} }\nclass Button implements First, Second {}\nfn main() {}",
            "E0212",
            "conflicting default method 'draw'",
            Some("implement 'draw' on 'Button' to resolve the conflict"),
        ),
        (
            "enum-unknown-variant.princi",
            "enum State { Ready }\nfn main() { let state = State.Missing }",
            "E0304",
            "enum 'State' has no variant 'Missing'",
            None,
        ),
        (
            "enum-payload-count.prnc",
            "enum State { Failed(String) }\nfn main() { let state = State.Failed() }",
            "E0304",
            "variant 'State.Failed' expects 1 payload value(s), found 0",
            None,
        ),
        (
            "enum-payload-type.princi",
            "enum State { Failed(Int) }\nfn main() { let state = State.Failed(\"offline\") }",
            "E0304",
            "payload 1 of variant 'State.Failed' expects Int, found String",
            None,
        ),
        (
            "non-exhaustive.princi",
            "enum State { Ready Waiting }\nfn main() { let state = State.Ready; match state { Ready => println(1) } }",
            "E0305",
            "missing variant: State.Waiting",
            None,
        ),
        (
            "unknown-pattern.prnc",
            "enum State { Ready Waiting }\nfn main() { let state = State.Ready; match state { Missing => println(1) } }",
            "E0304",
            "enum 'State' has no variant 'Missing'",
            None,
        ),
        (
            "unreachable-arm.prnc",
            "enum State { Ready }\nfn main() { let state = State.Ready; match state { _ => println(1) Ready => println(2) } }",
            "E0305",
            "unreachable match arm",
            None,
        ),
        (
            "match-result-type.princi",
            "fn main() { let value = match true { true => 1 false => \"no\" } }",
            "E0305",
            "match arm has type String, but earlier arms have type Int",
            None,
        ),
        (
            "result-question.prnc",
            "fn bad(value: Int) -> Int { return value? }\nfn main() {}",
            "E0306",
            "cannot apply '?' to Int; expected Result<T, E>",
            None,
        ),
        (
            "result-function-context.princi",
            "fn read() -> Result<Int, String> { return Result<Int, String>.Ok(1) }\nfn bad() -> Int { return read()? }\nfn main() {}",
            "E0306",
            "cannot propagate an error from a function returning Int",
            None,
        ),
        (
            "result-error-type.princi",
            "fn read() -> Result<Int, Int> { return Result<Int, Int>.Ok(1) }\nfn bad() -> Result<Int, String> { let value = read()?; return Result<Int, String>.Ok(value) }\nfn main() {}",
            "E0306",
            "cannot propagate error type Int; the enclosing function expects String",
            None,
        ),
        (
            "specialization.princi",
            "fn expand<T>(value: T) -> Int { let next = expand<List<T>>([value]); return next }\nfn main() { println(expand(1)) }",
            "E0307",
            "cannot be specialized finitely",
            None,
        ),
    ];

    for (name, program, code, message, note) in cases {
        let source = dir.0.join(name);
        fs::write(&source, program).expect("invalid Princi program should be written");
        let result = dir
            .compiler()
            .arg("build")
            .arg(&source)
            .output()
            .expect("compiler process should start");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "{name} unexpectedly compiled");
        assert_eq!(
            stderr.matches("error[E").count(),
            1,
            "{name}: redundant diagnostic cascade: {stderr}"
        );
        assert!(
            stderr.contains(&format!("error[{code}]")),
            "{name}: {stderr}"
        );
        assert!(stderr.contains(message), "{name}: {stderr}");
        assert!(
            stderr.contains(&format!("{name}:")),
            "{name}: missing location: {stderr}"
        );
        assert!(
            stderr.contains('^'),
            "{name}: missing source caret: {stderr}"
        );
        if let Some(note) = note {
            assert!(stderr.contains(note), "{name}: missing context: {stderr}");
        }
        assert!(!stderr.contains("LLVM"), "{name}: {stderr}");
        assert!(!stderr.contains("monomorph"), "{name}: {stderr}");
        assert!(!stderr.contains("panicked"), "{name}: {stderr}");
        assert!(
            !source.with_extension("exe").exists(),
            "{name} emitted an exe"
        );
    }
}
