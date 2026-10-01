mod options;
mod pipeline;

pub use options::{BuildOptions, CompilerOptions, Target};

use crate::diagnostics::Diagnostic;
use crate::source::SourceFile;

pub struct Compiler;

impl Compiler {
    pub fn build(options: &BuildOptions) -> Result<(), Diagnostic> {
        let source = SourceFile::load(&options.source_path)?;

        pipeline::compile(&source, options)
    }
}
