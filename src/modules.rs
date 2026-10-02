//! Resolution for the small, compiler-provided v0.1 module set.
//!
//! User files are deliberately not searched yet. This keeps imports independent
//! of the working directory and source extension while the language has no
//! project or package configuration.

use std::collections::HashSet;

use crate::ast::{Declaration, ImportDeclaration, Program};
use crate::diagnostics::Diagnostic;
use crate::source::SourceFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardModule {
    Io,
    Math,
}

impl StandardModule {
    pub fn name(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Math => "math",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "io" => Some(Self::Io),
            "math" => Some(Self::Math),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedModules {
    modules: Vec<StandardModule>,
}

impl ResolvedModules {
    pub fn modules(&self) -> &[StandardModule] {
        &self.modules
    }

    pub fn contains(&self, module: StandardModule) -> bool {
        self.modules.contains(&module)
    }
}

/// Resolve imports against the v0.1 standard-module registry.
///
/// Imports are identified by their canonical module name, never by the source
/// file extension. Repeated imports are idempotent and retain first-seen order.
pub fn resolve(source: &SourceFile, program: &Program) -> Result<ResolvedModules, Vec<Diagnostic>> {
    let mut resolved = ResolvedModules::default();
    let mut seen = HashSet::new();
    let mut diagnostics = Vec::new();

    for declaration in &program.declarations {
        let Declaration::Import(import) = declaration else {
            continue;
        };

        if import.path.len() != 1 {
            diagnostics.push(Diagnostic::at(
                "nested module imports are not supported in v0.1; import 'io' or 'math'",
                source.location(import.span),
            ));
            continue;
        }

        let name = import.path[0].name.as_str();
        let Some(module) = StandardModule::from_name(name) else {
            diagnostics.push(unknown_module_diagnostic(source, import, name));
            continue;
        };

        if seen.insert(module) {
            resolved.modules.push(module);
        }
    }

    if diagnostics.is_empty() {
        Ok(resolved)
    } else {
        Err(diagnostics)
    }
}

fn unknown_module_diagnostic(
    source: &SourceFile,
    import: &ImportDeclaration,
    name: &str,
) -> Diagnostic {
    let span = import
        .path
        .first()
        .map_or(import.span, |identifier| identifier.span);
    Diagnostic::at(
        format!("unknown standard module '{name}'; v0.1 supports 'io' and 'math'"),
        source.location(span),
    )
}

#[cfg(test)]
mod tests {
    use super::{resolve, ResolvedModules, StandardModule};
    use crate::codegen;
    use crate::lexer;
    use crate::parser;
    use crate::semantic;
    use crate::source::SourceFile;

    fn parse(source: &SourceFile) -> parser::ParseResult {
        parser::parse(
            source,
            lexer::lex(source).expect("module test source should lex"),
        )
    }

    #[test]
    fn resolves_standard_imports_from_a_real_source_program_with_both_extensions() {
        let text = include_str!("../examples/modules.prnc");
        let mut resolved_for_extensions = Vec::new();

        for path in ["modules.prnc", "modules.princi"] {
            let source = SourceFile::from_text(path, text);
            let parsed = parse(&source);
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

            let modules = resolve(&source, &parsed.program).expect("standard imports resolve");
            assert_eq!(
                modules.modules(),
                &[StandardModule::Io, StandardModule::Math]
            );

            let analyzed = semantic::analyze_with_modules(&source, &parsed.program, &modules);
            assert!(
                analyzed.diagnostics.is_empty(),
                "{:?}",
                analyzed.diagnostics
            );

            let ir = codegen::generate_llvm_ir(&analyzed.typed_program, &source)
                .expect("the imported source should reach LLVM generation");
            assert!(ir.contains("@princi_fn_"));
            resolved_for_extensions.push(modules);
        }

        assert_eq!(resolved_for_extensions[0], resolved_for_extensions[1]);
    }

    #[test]
    fn duplicate_imports_are_loaded_once_in_first_seen_order() {
        let source = SourceFile::from_text(
            "duplicates.prnc",
            "import io\nimport math\nimport io\nfn main() {}\n",
        );
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

        let modules = resolve(&source, &parsed.program).expect("known modules resolve");
        assert_eq!(
            modules,
            ResolvedModules {
                modules: vec![StandardModule::Io, StandardModule::Math]
            }
        );
    }

    #[test]
    fn unknown_import_reports_a_source_positioned_diagnostic() {
        let source = SourceFile::from_text("unknown.prnc", "import graphics\nfn main() {}\n");
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

        let diagnostics = resolve(&source, &parsed.program).expect_err("unknown module rejected");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0]
            .to_string()
            .contains("unknown.prnc:1:8: error: unknown standard module 'graphics'"));
    }

    #[test]
    fn nested_import_paths_are_rejected_without_filesystem_resolution() {
        let source = SourceFile::from_text("nested.prnc", "import io.fs\nfn main() {}\n");
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

        let diagnostics = resolve(&source, &parsed.program).expect_err("nested module rejected");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0]
            .to_string()
            .contains("nested.prnc:1:1: error: nested module imports are not supported"));
    }

    #[test]
    fn path_traversal_syntax_is_rejected_by_the_import_parser() {
        let source = SourceFile::from_text("traversal.prnc", "import ../outside\nfn main() {}\n");
        let parsed = parse(&source);
        assert!(!parsed.diagnostics.is_empty());
        assert!(parsed.diagnostics[0]
            .to_string()
            .starts_with("traversal.prnc:1:8: error:"));
    }
}
