use crate::compiler::BuildOptions;
use crate::diagnostics::Diagnostic;
use crate::lexer;
use crate::source::SourceFile;

pub fn compile(source: &SourceFile, _options: &BuildOptions) -> Result<(), Diagnostic> {
    let _tokens = lexer::lex(source)?;

    // Parser and later stages are still under development.
    Err(Diagnostic::pipeline_stage_incomplete("parser"))
}
