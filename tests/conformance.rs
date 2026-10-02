use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("princi conformance-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("temporary conformance directory should be created");
        Self(path)
    }

    fn source(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, text).expect("fixture source should be written");
        path
    }

    fn build(&self, source: &Path, output: Option<&Path>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_princi"));
        command.arg("build").arg(source);
        if let Some(output) = output {
            command.arg("-o").arg(output);
        }
        command.output().expect("compiler process should start")
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn normalize_newlines(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

fn reject_fixture(name: &str, fixture: &str, code: &str, custom_output: bool) {
    let dir = TestDir::new();
    let source = dir.source(name, fixture);
    let output = custom_output.then(|| dir.0.join("custom output.exe"));
    let result = dir.build(&source, output.as_deref());
    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(
        !result.status.success(),
        "{name} must fail to compile: {stderr}"
    );
    assert!(stderr.contains(&format!("error[{code}]")), "{stderr}");
    assert!(
        !stderr.contains("panicked"),
        "user errors must not panic: {stderr}"
    );
    assert!(
        !source.with_extension("exe").exists(),
        "failed build emitted an executable at {}",
        source.with_extension("exe").display()
    );
    if let Some(output) = output {
        assert!(
            !output.exists(),
            "failed build emitted {}",
            output.display()
        );
    }
}

#[test]
fn negative_fixtures_report_diagnostics_and_never_emit_executables() {
    let mismatch_prnc = include_str!("fixtures/invalid/type_mismatch.prnc");
    let mismatch_princi = include_str!("fixtures/invalid/type_mismatch.princi");
    assert_eq!(mismatch_prnc, mismatch_princi, "extensions are aliases");
    reject_fixture("type mismatch.prnc", mismatch_prnc, "E0201", false);
    reject_fixture("type mismatch.princi", mismatch_princi, "E0201", false);

    reject_fixture(
        "heterogeneous list.prnc",
        include_str!("fixtures/invalid/heterogeneous_list.prnc"),
        "E0299",
        false,
    );
    reject_fixture(
        "unknown import.prnc",
        include_str!("fixtures/invalid/unknown_import.prnc"),
        "E0209",
        false,
    );
    reject_fixture(
        "missing main.prnc",
        include_str!("fixtures/invalid/missing_main.prnc"),
        "E0207",
        true,
    );
    reject_fixture(
        "unsafe ffi.prnc",
        include_str!("fixtures/invalid/unsafe_ffi.prnc"),
        "E0299",
        false,
    );

    for (name, fixture, code) in [
        (
            "unknown identifier.prnc",
            include_str!("fixtures/invalid/unknown_identifier.prnc"),
            "E0202",
        ),
        (
            "unknown type.prnc",
            include_str!("fixtures/invalid/unknown_type.prnc"),
            "E0203",
        ),
        (
            "incorrect arguments.prnc",
            include_str!("fixtures/invalid/incorrect_arguments.prnc"),
            "E0204",
        ),
        (
            "invalid member.prnc",
            include_str!("fixtures/invalid/invalid_member.prnc"),
            "E0205",
        ),
        (
            "duplicate declaration.prnc",
            include_str!("fixtures/invalid/duplicate_declaration.prnc"),
            "E0206",
        ),
        (
            "invalid return.prnc",
            include_str!("fixtures/invalid/invalid_return.prnc"),
            "E0208",
        ),
        (
            "invalid syntax.prnc",
            include_str!("fixtures/invalid/invalid_syntax.prnc"),
            "E0101",
        ),
    ] {
        reject_fixture(name, fixture, code, false);
    }
}

#[test]
fn positive_conformance_fixtures_keep_both_suffixes_as_exact_aliases() {
    assert_eq!(
        include_str!("fixtures/conformance/v0_1.prnc"),
        include_str!("fixtures/conformance/v0_1.princi"),
        ".prnc and .princi fixtures must contain the same program"
    );
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
        return false;
    }

    let reason = "native conformance needs LLVM/Clang with X86 support and x86-64 MinGW-w64 GCC";
    if std::env::var_os("PRINCI_REQUIRE_NATIVE_TESTS").is_some() {
        panic!("{reason}; PRINCI_REQUIRE_NATIVE_TESTS is set");
    }
    eprintln!("skipping Windows native conformance: {reason}");
    true
}

#[cfg(windows)]
#[test]
fn complete_v0_1_program_builds_and_runs_identically_with_both_extensions() {
    if skip_without_native_toolchain() {
        return;
    }

    let prnc = include_str!("fixtures/conformance/v0_1.prnc");
    let princi = include_str!("fixtures/conformance/v0_1.princi");
    assert_eq!(
        prnc, princi,
        ".prnc and .princi must contain the same program"
    );
    let expected_stdout = include_str!("fixtures/conformance/v0_1.stdout");
    let expected_stderr = include_str!("fixtures/conformance/v0_1.stderr");
    let expected_exit = include_str!("fixtures/conformance/v0_1.exitcode")
        .trim()
        .parse::<i32>()
        .expect("fixture exit code should be an integer");

    for (extension, source_text) in [("prnc", prnc), ("princi", princi)] {
        let dir = TestDir::new();
        let source = dir.source(&format!("conformance.{extension}"), source_text);

        // Intentionally omit -o: the default output must be next to the input
        // and use its filename with an .exe extension.
        let build = dir.build(&source, None);
        assert!(
            build.status.success(),
            "{extension} build failed: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(build.stdout.is_empty(), "successful build should be quiet");
        assert!(
            build.stderr.is_empty(),
            "successful build should not emit diagnostics: {}",
            String::from_utf8_lossy(&build.stderr)
        );

        let executable = source.with_extension("exe");
        assert!(
            executable.is_file(),
            "build should produce {}",
            executable.display()
        );
        let run = Command::new(&executable)
            .output()
            .expect("generated Windows executable should launch");
        assert_eq!(
            normalize_newlines(&run.stdout),
            expected_stdout,
            "unexpected stdout for .{extension}"
        );
        assert_eq!(
            normalize_newlines(&run.stderr),
            expected_stderr,
            "unexpected stderr for .{extension}"
        );
        assert_eq!(run.status.code(), Some(expected_exit));
    }
}
