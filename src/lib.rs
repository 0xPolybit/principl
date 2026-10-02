pub mod ast;
pub mod cli;
pub mod codegen;
pub mod compiler;
pub mod diagnostics;
pub mod ffi;
pub mod lexer;
pub mod modules;
pub mod parser;
pub mod runtime;
pub mod semantic;
pub mod source;
pub mod types;

use std::ffi::OsString;

use crate::cli::parse_build_args;
use crate::compiler::{BuildOptions, Compiler};
use crate::diagnostics::DiagnosticBundle;

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), DiagnosticBundle> {
    let parsed = parse_build_args(args).map_err(DiagnosticBundle::from)?;
    let options = BuildOptions::from_parsed(parsed).map_err(DiagnosticBundle::from)?;
    Compiler::build(&options)
}
