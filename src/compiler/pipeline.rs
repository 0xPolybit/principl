use crate::codegen;
use crate::compiler::BuildOptions;
use crate::diagnostics::DiagnosticBundle;
use crate::lexer;
use crate::modules;
use crate::parser;
use crate::semantic;
use crate::source::SourceFile;

pub fn compile(source: &SourceFile, options: &BuildOptions) -> Result<(), DiagnosticBundle> {
    let tokens = lexer::lex(source).map_err(DiagnosticBundle::from)?;
    let parsed = parser::parse(source, tokens);
    if !parsed.diagnostics.is_empty() {
        return Err(DiagnosticBundle::from_diagnostics(parsed.diagnostics));
    }

    let resolved_modules =
        modules::resolve(source, &parsed.program).map_err(DiagnosticBundle::from_diagnostics)?;
    let analyzed = semantic::analyze_with_modules(source, &parsed.program, &resolved_modules);
    if !analyzed.diagnostics.is_empty() {
        return Err(DiagnosticBundle::from_diagnostics(analyzed.diagnostics));
    }

    let llvm_ir = codegen::generate_llvm_ir(&analyzed.typed_program, source)
        .map_err(DiagnosticBundle::from)?;
    codegen::compile_native(&llvm_ir, options).map_err(DiagnosticBundle::from)
}
