use crate::compiler::BuildOptions;
use crate::diagnostics::{Diagnostic, DiagnosticBundle};
use crate::lexer;
use crate::parser;
use crate::semantic;
use crate::source::SourceFile;

pub fn compile(source: &SourceFile, _options: &BuildOptions) -> Result<(), DiagnosticBundle> {
    let tokens = lexer::lex(source).map_err(DiagnosticBundle::from)?;
    let parsed = parser::parse(source, tokens);
    if !parsed.diagnostics.is_empty() {
        return Err(DiagnosticBundle::from_diagnostics(parsed.diagnostics));
    }

    let analyzed = semantic::analyze(source, &parsed.program);
    if !analyzed.diagnostics.is_empty() {
        return Err(DiagnosticBundle::from_diagnostics(analyzed.diagnostics));
    }

    // Typed IR lowering and later stages are still under development.
    Err(Diagnostic::pipeline_stage_incomplete("code generation").into())
}
