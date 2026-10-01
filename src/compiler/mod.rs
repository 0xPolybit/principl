mod options;
mod pipeline;

pub use options::{BuildOptions, CompilerOptions, Target};

use crate::diagnostics::DiagnosticBundle;
use crate::source::SourceFile;

pub struct Compiler;

impl Compiler {
    pub fn build(options: &BuildOptions) -> Result<(), DiagnosticBundle> {
        let source = SourceFile::load(&options.source_path).map_err(DiagnosticBundle::from)?;

        pipeline::compile(&source, options)
    }
}
