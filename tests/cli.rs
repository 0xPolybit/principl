use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
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
        eprintln!(
            "skipping Windows executable integration check: install LLVM/Clang with IR and X86 support and x86-64 MinGW-w64 GCC"
        );
        return;
    }
    let dir = TestDir::new();

    for name in ["hello.prnc", "hello.princi"] {
        let source = dir.source(name);
        fs::write(&source, "fn main() { println(1 + 2 * 3) }")
            .expect("valid Princi program should be written");
        let output = source.with_extension("exe");
        let result = Command::new(env!("CARGO_BIN_EXE_princi"))
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
    let result = Command::new(env!("CARGO_BIN_EXE_princi"))
        .arg("build")
        .arg(&source)
        .env("PRINCI_CLANG", missing_clang)
        .output()
        .expect("CLI process should start");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("could not start LLVM IR compiler (clang)"));
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
        eprintln!("skipping linker diagnostic check because LLVM/Clang is unavailable");
        return;
    }

    let dir = TestDir::new();
    let source = dir.0.join("valid.princi");
    fs::write(&source, "fn main() {}").expect("valid source should be written");
    let missing_linker = dir.0.join("missing MinGW linker.exe");
    let result = Command::new(env!("CARGO_BIN_EXE_princi"))
        .arg("build")
        .arg(&source)
        .env("PRINCI_CC", missing_linker)
        .output()
        .expect("CLI process should start");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("could not start Windows linker"));
    assert!(stderr.contains("install x86-64 MinGW GCC"));
}

#[test]
fn unsupported_extensions_and_missing_sources_exit_nonzero() {
    let dir = TestDir::new();
    let unsupported = dir.source("hello.txt");

    for source in [unsupported, dir.0.join("missing.prnc")] {
        let result = Command::new(env!("CARGO_BIN_EXE_princi"))
            .arg("build")
            .arg(source)
            .output()
            .expect("CLI process should start");
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("princi: error:"),
            "CLI failure should include a useful diagnostic"
        );
    }
}

#[test]
fn malformed_source_reports_its_file_line_and_column() {
    let dir = TestDir::new();
    let source = dir.0.join("hello.prnc");
    fs::write(
        &source,
        "fn main() {\n  let value = 1;\n  let other = 2;\n           \"unfinished\n}\n",
    )
    .expect("malformed source should be written");
    let expected_path = fs::canonicalize(&source).expect("source should canonicalize");

    let result = Command::new(env!("CARGO_BIN_EXE_princi"))
        .arg("build")
        .arg(&source)
        .output()
        .expect("CLI process should start");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains(&format!(
            "{}:4:12: error: unterminated string literal",
            expected_path.display()
        )),
        "expected located lexical diagnostic, got: {stderr}"
    );
}

#[test]
fn semantic_errors_are_reported_before_later_pipeline_stages() {
    let dir = TestDir::new();
    let source = dir.0.join("type_error.princi");
    fs::write(&source, "fn main() {\n    let count: Int = \"wrong\"\n}")
        .expect("source with a type error should be written");
    let expected_path = fs::canonicalize(&source).expect("source should canonicalize");

    let result = Command::new(env!("CARGO_BIN_EXE_princi"))
        .arg("build")
        .arg(&source)
        .output()
        .expect("CLI process should start");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains(&format!(
            "{}:2:22: error: expression expects Int, found String",
            expected_path.display()
        )),
        "expected a located semantic diagnostic, got: {stderr}"
    );
    assert!(
        !stderr.contains("code generation stage"),
        "semantic failures must stop before later pipeline stages"
    );
}
