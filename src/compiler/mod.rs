mod options;
mod pipeline;

pub use options::{BuildOptions, CompilerOptions, Target};

use std::fs;

use crate::diagnostics::Diagnostic;

pub struct Compiler;

impl Compiler {
    pub fn build(options: &BuildOptions) -> Result<(), Diagnostic> {
        let source = fs::read_to_string(&options.source_path).map_err(|error| {
            Diagnostic::new(format!(
                "could not read source file '{}': {error}",
                options.source_path.display()
            ))
        })?;

        pipeline::compile(&source, options)
    }
}
