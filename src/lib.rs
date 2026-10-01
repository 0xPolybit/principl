pub mod cli;
pub mod compiler;
pub mod diagnostics;
pub mod lexer;
pub mod source;

use std::ffi::OsString;

use crate::cli::parse_build_args;
use crate::compiler::{BuildOptions, Compiler};
use crate::diagnostics::Diagnostic;

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), Diagnostic> {
    let parsed = parse_build_args(args)?;
    let options = BuildOptions::from_parsed(parsed)?;
    Compiler::build(&options)
}
