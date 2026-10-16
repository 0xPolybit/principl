use crate::ast::Program;
use crate::codegen;
use crate::compiler::BuildOptions;
use crate::diagnostics::DiagnosticBundle;
use crate::lexer;
use crate::modules;
use crate::parser;
use crate::prelude;
use crate::semantic;
use crate::source::SourceFile;

pub fn compile(source: &SourceFile, options: &BuildOptions) -> Result<(), DiagnosticBundle> {
    let (compilation_source, program) = compilation_unit(source)?;

    let resolved_modules = modules::resolve(&compilation_source, &program)
        .map_err(DiagnosticBundle::from_diagnostics)?;
    let analyzed = semantic::analyze_with_modules(&compilation_source, &program, &resolved_modules);
    if !analyzed.diagnostics.is_empty() {
        return Err(DiagnosticBundle::from_diagnostics(analyzed.diagnostics));
    }

    let llvm_ir = codegen::generate_llvm_ir(&analyzed.typed_program, &compilation_source)
        .map_err(DiagnosticBundle::from)?;
    codegen::compile_native(&llvm_ir, options).map_err(DiagnosticBundle::from)
}

fn compilation_unit(source: &SourceFile) -> Result<(SourceFile, Program), DiagnosticBundle> {
    let user_program = parse_program(source)?;
    if prelude::declares_result(&user_program) {
        return Ok((source.clone(), user_program));
    }

    let source_with_prelude = prelude::append_source(source);
    let program = parse_program(&source_with_prelude)?;
    Ok((source_with_prelude, program))
}

fn parse_program(source: &SourceFile) -> Result<Program, DiagnosticBundle> {
    let tokens = lexer::lex(source).map_err(DiagnosticBundle::from)?;
    let parsed = parser::parse(source, tokens);
    if parsed.diagnostics.is_empty() {
        Ok(parsed.program)
    } else {
        Err(DiagnosticBundle::from_diagnostics(parsed.diagnostics))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Declaration;
    use crate::source::SourceFile;

    use super::compilation_unit;

    #[test]
    fn appends_result_as_parseable_source_prelude() {
        let user_source = SourceFile::from_text("hello.prnc", "fn main() {}\n");
        let (combined_source, program) =
            compilation_unit(&user_source).expect("the prelude should parse normally");

        assert!(combined_source.text().starts_with(user_source.text()));
        assert_eq!(combined_source.path(), user_source.path());
        assert_eq!(
            program
                .declarations
                .iter()
                .filter(|declaration| matches!(declaration, Declaration::Enum(enumeration) if enumeration.name.name == "Result"))
                .count(),
            1
        );
    }

    #[test]
    fn keeps_a_user_declared_result_instead_of_adding_the_prelude_copy() {
        let source = SourceFile::from_text(
            "custom.princi",
            "enum Result<T, E> { Success(T) Failure(E) }\nfn main() {}\n",
        );
        let (compilation_source, program) =
            compilation_unit(&source).expect("the user's enum should parse normally");

        assert_eq!(compilation_source.text(), source.text());
        assert_eq!(
            program
                .declarations
                .iter()
                .filter(|declaration| matches!(declaration, Declaration::Enum(enumeration) if enumeration.name.name == "Result"))
                .count(),
            1
        );
    }
}
