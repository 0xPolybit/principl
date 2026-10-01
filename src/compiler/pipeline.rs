use crate::compiler::BuildOptions;
use crate::diagnostics::Diagnostic;

pub fn compile(_source: &str, _options: &BuildOptions) -> Result<(), Diagnostic> {
    // The CLI now enters the compilation pipeline here. Language grammar and
    // frontend/code-generation stages are intentionally not invented by the
    // CLI implementation task.
    Err(Diagnostic::pipeline_stage_incomplete("lexer"))
}
