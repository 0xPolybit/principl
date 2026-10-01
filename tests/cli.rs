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
            "princi-cli-integration-{}-{id}",
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

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn both_extensions_reach_the_compiler_pipeline() {
    let dir = TestDir::new();

    for name in ["hello.prnc", "hello.princi"] {
        let source = dir.source(name);
        let output = dir.0.join("program.exe");
        let result = Command::new(env!("CARGO_BIN_EXE_princi"))
            .arg("build")
            .arg(&source)
            .arg("-o")
            .arg(&output)
            .output()
            .expect("CLI process should start");

        assert!(!result.status.success(), "incomplete pipeline must fail");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains("the lexer stage is not implemented yet"),
            "expected a pipeline diagnostic for {name}, got: {stderr}"
        );
        assert!(
            !stderr.contains("unsupported source extension"),
            "{name} should be accepted by extension validation"
        );
        assert!(
            !output.exists(),
            "incomplete compilation must not claim output"
        );
    }
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
