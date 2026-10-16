use crate::ast::{Declaration, Program};
use crate::source::SourceFile;

pub(crate) const SOURCE: &str = include_str!("../stdlib/prelude.princi");

/// The source-level prelude name can be replaced by a declaration in the
/// user's compilation unit. Keeping this rule at the source boundary lets the
/// ordinary enum semantic and code-generation paths handle either definition.
pub(crate) fn declares_result(program: &Program) -> bool {
    program
        .declarations
        .iter()
        .any(|declaration| match declaration {
            Declaration::Function(function) => function.name.name == "Result",
            Declaration::Class(declaration) | Declaration::Struct(declaration) => {
                declaration.name.name == "Result"
            }
            Declaration::Enum(declaration) => declaration.name.name == "Result",
            Declaration::Interface(declaration) => declaration.name.name == "Result",
            Declaration::Trait(declaration) => declaration.name.name == "Result",
            Declaration::ExternBlock(block) => block
                .functions
                .iter()
                .any(|function| function.name.name == "Result"),
            Declaration::Import(_) => false,
        })
}

pub(crate) fn append_source(source: &SourceFile) -> SourceFile {
    let mut text = String::with_capacity(source.text().len() + SOURCE.len() + 2);
    text.push_str(source.text());
    text.push_str("\n\n");
    text.push_str(SOURCE);
    SourceFile::from_text(source.path().to_path_buf(), text)
}
