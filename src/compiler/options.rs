use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::cli::ParsedBuildArgs;
use crate::diagnostics::{Diagnostic, DiagnosticCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    WindowsX86_64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerOptions {
    pub target: Target,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: Target::WindowsX86_64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub compiler: CompilerOptions,
}

impl BuildOptions {
    pub fn from_parsed(parsed: ParsedBuildArgs) -> Result<Self, Diagnostic> {
        validate_extension(&parsed.source)?;

        let source_path = fs::canonicalize(&parsed.source).map_err(|error| {
            let message = if error.kind() == std::io::ErrorKind::NotFound {
                format!("source file '{}' was not found", parsed.source.display())
            } else {
                format!(
                    "could not resolve source file '{}': {error}",
                    parsed.source.display()
                )
            };
            Diagnostic::coded(DiagnosticCode::SourceFile, message)
        })?;

        if !source_path.is_file() {
            return Err(Diagnostic::coded(
                DiagnosticCode::SourceFile,
                format!("source path '{}' is not a file", parsed.source.display()),
            ));
        }

        let output_path = match parsed.output {
            Some(path) => normalize_output_path(&path)?,
            None => source_path.with_extension("exe"),
        };

        Ok(Self {
            source_path,
            output_path,
            compiler: CompilerOptions::default(),
        })
    }
}

fn validate_extension(path: &Path) -> Result<(), Diagnostic> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    if extension.eq_ignore_ascii_case("prnc") || extension.eq_ignore_ascii_case("princi") {
        Ok(())
    } else {
        let display_extension = if extension.is_empty() {
            "<none>"
        } else {
            extension
        };
        Err(Diagnostic::unsupported_extension(display_extension))
    }
}

fn normalize_output_path(path: &Path) -> Result<PathBuf, Diagnostic> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| {
                Diagnostic::coded(
                    DiagnosticCode::BuildOutput,
                    format!("could not resolve current directory: {error}"),
                )
            })?
            .join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.file_name().is_some() {
                    normalized.pop();
                }
            }
        }
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::{BuildOptions, CompilerOptions, Target};
    use crate::cli::ParsedBuildArgs;
    use crate::diagnostics::DiagnosticCode;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("princi-cli-test-{}-{id}", std::process::id()));
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
    fn accepts_both_extensions_and_derives_the_same_executable_name() {
        let dir = TestDir::new();
        for name in ["hello.prnc", "hello.princi"] {
            let source = dir.source(name);
            let options = BuildOptions::from_parsed(ParsedBuildArgs {
                source: source.clone(),
                output: None,
            })
            .expect("both source extensions should validate");

            assert_eq!(
                options.output_path,
                fs::canonicalize(&source)
                    .expect("source should normalize")
                    .with_extension("exe")
            );
            assert_eq!(
                options.compiler,
                CompilerOptions {
                    target: Target::WindowsX86_64
                }
            );
        }
    }

    #[test]
    fn rejects_unsupported_extensions_and_missing_sources() {
        let dir = TestDir::new();
        let unsupported = dir.source("hello.txt");
        let error = BuildOptions::from_parsed(ParsedBuildArgs {
            source: unsupported,
            output: None,
        })
        .expect_err("unsupported extension should fail");
        assert!(error.message().contains("expected .prnc or .princi"));

        let missing = dir.0.join("missing.prnc");
        let error = BuildOptions::from_parsed(ParsedBuildArgs {
            source: missing,
            output: None,
        })
        .expect_err("missing source should fail");
        assert!(error.message().contains("source file '"));
        assert_eq!(error.code(), DiagnosticCode::SourceFile);
    }

    #[test]
    fn normalizes_input_and_explicit_output_paths() {
        let dir = TestDir::new();
        let source = dir.source("hello.princi");
        fs::create_dir(dir.0.join("nested")).expect("nested directory should be created");
        let input = dir.0.join("nested").join("..").join("hello.princi");
        let output = Path::new("build").join("..").join("program.exe");
        let expected_output = std::env::current_dir()
            .expect("current directory should be available")
            .join("program.exe");

        let options = BuildOptions::from_parsed(ParsedBuildArgs {
            source: input,
            output: Some(output),
        })
        .expect("normalized paths should validate");

        assert_eq!(options.source_path, fs::canonicalize(source).unwrap());
        assert_eq!(options.output_path, expected_output);
    }
}
