use std::ffi::OsString;
use std::path::PathBuf;

use crate::diagnostics::Diagnostic;

const USAGE: &str = "usage: princi build <source-file> [-o <output-file>]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBuildArgs {
    pub source: PathBuf,
    pub output: Option<PathBuf>,
}

pub fn parse_build_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<ParsedBuildArgs, Diagnostic> {
    let mut args = args.into_iter();
    let command = args.next().ok_or_else(|| Diagnostic::new(USAGE))?;

    if command != "build" {
        return Err(Diagnostic::new(format!(
            "unknown command '{}'; {USAGE}",
            command.to_string_lossy()
        )));
    }

    let mut source = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        if arg == "-o" {
            if output.is_some() {
                return Err(Diagnostic::new("-o may only be specified once"));
            }

            let path = args
                .next()
                .ok_or_else(|| Diagnostic::new("-o requires an output path"))?;
            if path.is_empty() {
                return Err(Diagnostic::new("-o requires a non-empty output path"));
            }
            output = Some(PathBuf::from(path));
        } else if arg.to_string_lossy().starts_with('-') {
            return Err(Diagnostic::new(format!(
                "unknown build option '{}'; {USAGE}",
                arg.to_string_lossy()
            )));
        } else if source.is_none() {
            source = Some(PathBuf::from(arg));
        } else {
            return Err(Diagnostic::new(format!(
                "unexpected argument '{}'; {USAGE}",
                arg.to_string_lossy()
            )));
        }
    }

    let source = source.ok_or_else(|| Diagnostic::new(USAGE))?;

    Ok(ParsedBuildArgs { source, output })
}

#[cfg(test)]
mod tests {
    use super::parse_build_args;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_build_source_and_optional_output() {
        let parsed = parse_build_args(args(&["build", "hello.prnc", "-o", "program.exe"]))
            .expect("valid build arguments should parse");

        assert_eq!(parsed.source, PathBuf::from("hello.prnc"));
        assert_eq!(parsed.output, Some(PathBuf::from("program.exe")));
    }

    #[test]
    fn accepts_both_source_suffixes_in_argument_parser() {
        for source in ["hello.prnc", "hello.princi"] {
            let parsed = parse_build_args(args(&["build", source]))
                .expect("both Princi extensions should parse");
            assert_eq!(parsed.source, PathBuf::from(source));
        }
    }

    #[test]
    fn rejects_other_commands_and_incomplete_options() {
        assert!(parse_build_args(args(&["run", "hello.prnc"])).is_err());
        assert!(parse_build_args(args(&["build", "hello.prnc", "-o"])).is_err());
    }
}
