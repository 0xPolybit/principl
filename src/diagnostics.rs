use std::fmt;
use std::path::Path;

use crate::source::{SourceFile, SourceLocation, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    CommandLine,
    UnsupportedExtension,
    SourceFile,
    Lexical,
    Syntax,
    TypeMismatch,
    UnknownIdentifier,
    UnknownType,
    InvalidCall,
    InvalidMember,
    DuplicateDeclaration,
    MissingMain,
    InvalidReturn,
    UnknownModule,
    UninitializedVariable,
    InvalidInterface,
    InvalidTrait,
    GenericConstraint,
    Semantic,
    UnsupportedFeature,
    GenericArgument,
    GenericInference,
    EnumPattern,
    MatchAnalysis,
    ResultPropagation,
    GenericSpecialization,
    InternalCompiler,
    LlvmUnavailable,
    LlvmFailure,
    LinkerUnavailable,
    LinkerFailure,
    BuildOutput,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommandLine => "E0001",
            Self::UnsupportedExtension => "E0002",
            Self::SourceFile => "E0003",
            Self::Lexical => "E0100",
            Self::Syntax => "E0101",
            Self::TypeMismatch => "E0201",
            Self::UnknownIdentifier => "E0202",
            Self::UnknownType => "E0203",
            Self::InvalidCall => "E0204",
            Self::InvalidMember => "E0205",
            Self::DuplicateDeclaration => "E0206",
            Self::MissingMain => "E0207",
            Self::InvalidReturn => "E0208",
            Self::UnknownModule => "E0209",
            Self::UninitializedVariable => "E0210",
            Self::InvalidInterface => "E0211",
            Self::InvalidTrait => "E0212",
            Self::GenericConstraint => "E0213",
            Self::Semantic => "E0299",
            Self::UnsupportedFeature => "E0301",
            Self::GenericArgument => "E0302",
            Self::GenericInference => "E0303",
            Self::EnumPattern => "E0304",
            Self::MatchAnalysis => "E0305",
            Self::ResultPropagation => "E0306",
            Self::GenericSpecialization => "E0307",
            Self::InternalCompiler => "E9001",
            Self::LlvmUnavailable => "E0401",
            Self::LlvmFailure => "E9002",
            Self::LinkerUnavailable => "E0403",
            Self::LinkerFailure => "E0404",
            Self::BuildOutput => "E0405",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::InternalCompiler => "internal compiler error",
            Self::LlvmFailure => "compiler error",
            Self::LlvmUnavailable
            | Self::LinkerUnavailable
            | Self::LinkerFailure
            | Self::BuildOutput => "toolchain error",
            _ => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceExcerpt {
    line: String,
    caret_column: usize,
    caret_width: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    message: String,
    location: Option<SourceLocation>,
    excerpt: Option<SourceExcerpt>,
    notes: Vec<String>,
}

impl Diagnostic {
    /// Construct a command-line/build input error.
    pub fn new(message: impl Into<String>) -> Self {
        Self::coded(DiagnosticCode::CommandLine, message)
    }

    pub fn coded(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            excerpt: None,
            notes: Vec::new(),
        }
    }

    /// Compatibility constructor for a located diagnostic without source text.
    pub fn at(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::at_location(DiagnosticCode::Semantic, message, location)
    }

    pub fn at_location(
        code: DiagnosticCode,
        message: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            location: Some(location),
            excerpt: None,
            notes: Vec::new(),
        }
    }

    pub fn at_source(
        code: DiagnosticCode,
        message: impl Into<String>,
        source: &SourceFile,
        span: SourceSpan,
    ) -> Self {
        let (line, caret_column, caret_width) = source.excerpt(span);
        Self {
            code,
            message: message.into(),
            location: Some(source.location(span)),
            excerpt: Some(SourceExcerpt {
                line,
                caret_column,
                caret_width,
            }),
            notes: Vec::new(),
        }
    }

    /// Attach concise secondary context without expanding the primary error.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn unsupported_extension(extension: &str) -> Self {
        Self::coded(
            DiagnosticCode::UnsupportedExtension,
            format!("unsupported source extension '{extension}'; expected .prnc or .princi"),
        )
    }

    pub fn code(&self) -> DiagnosticCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn location(&self) -> Option<&SourceLocation> {
        self.location.as_ref()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = self.code.label();
        if let Some(location) = &self.location {
            let file = diagnostic_path(&location.file);
            write!(
                f,
                "{}:{}:{}: {label}[{}]: {}",
                file,
                location.line,
                location.column,
                self.code.as_str(),
                self.message
            )?;
            if let Some(excerpt) = &self.excerpt {
                write!(
                    f,
                    "\n\n    {}\n    {}{}",
                    excerpt.line,
                    " ".repeat(excerpt.caret_column),
                    "^".repeat(excerpt.caret_width)
                )?;
            }
        } else {
            write!(
                f,
                "princi: {label}[{}]: {}",
                self.code.as_str(),
                self.message
            )?;
        }
        for note in &self.notes {
            write!(f, "\n\n    = note: {note}")?;
        }
        Ok(())
    }
}

fn diagnostic_path(path: &Path) -> String {
    let rendered = path.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(unc_path) = rendered.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{unc_path}");
        }
        if let Some(path) = rendered.strip_prefix(r"\\?\") {
            return path.to_owned();
        }
    }
    rendered.into_owned()
}

impl std::error::Error for Diagnostic {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticBundle {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBundle {
    pub fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl From<Diagnostic> for DiagnosticBundle {
    fn from(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }
}

impl fmt::Display for DiagnosticBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}

impl std::error::Error for DiagnosticBundle {}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticCode};
    use crate::source::{SourceFile, SourceSpan};

    #[test]
    fn formats_located_diagnostics_with_code_excerpt_and_span() {
        let source = SourceFile::from_text(
            "hello.prnc",
            "fn main() {\n    var age: Int = \"twenty\"\n}",
        );
        let start = source
            .text()
            .find("\"twenty\"")
            .expect("literal should exist");
        let span = SourceSpan::new(start, start + "\"twenty\"".len());
        let error = Diagnostic::at_source(
            DiagnosticCode::TypeMismatch,
            "expected Int, found String",
            &source,
            span,
        );

        assert_eq!(
            error.to_string(),
            format!(
                "hello.prnc:2:20: error[E0201]: expected Int, found String\n\n        var age: Int = \"twenty\"\n    {}^^^^^^^^",
                " ".repeat(19)
            )
        );
        assert_eq!(error.code(), DiagnosticCode::TypeMismatch);
        assert_eq!(
            error.location().expect("location should be retained").span,
            span
        );
    }

    #[test]
    fn identifies_internal_and_toolchain_failures_separately() {
        assert_eq!(
            Diagnostic::coded(DiagnosticCode::InternalCompiler, "unexpected failure").to_string(),
            "princi: internal compiler error[E9001]: unexpected failure"
        );
        assert_eq!(
            Diagnostic::coded(DiagnosticCode::LlvmUnavailable, "install LLVM/Clang").to_string(),
            "princi: toolchain error[E0401]: install LLVM/Clang"
        );
    }

    #[test]
    fn formats_unlocated_input_diagnostics() {
        assert_eq!(
            Diagnostic::new("missing input").to_string(),
            "princi: error[E0001]: missing input"
        );
    }

    #[test]
    fn formats_secondary_notes_without_changing_the_primary_source_excerpt() {
        let source = SourceFile::from_text("example.prnc", "show(value)");
        let span = SourceSpan::new(5, 10);
        let error = Diagnostic::at_source(
            DiagnosticCode::GenericConstraint,
            "type Int does not satisfy trait Printable",
            &source,
            span,
        )
        .with_note("required by fn show<T: Printable>");

        assert!(error
            .to_string()
            .contains("example.prnc:1:6: error[E0213]: type Int does not satisfy trait Printable"));
        assert!(error
            .to_string()
            .ends_with("    = note: required by fn show<T: Printable>"));
    }

    #[test]
    fn generated_native_code_failures_do_not_expose_llvm_as_the_user_error() {
        assert_eq!(
            Diagnostic::coded(
                DiagnosticCode::LlvmFailure,
                "the compiler could not generate a valid native executable"
            )
            .to_string(),
            "princi: compiler error[E9002]: the compiler could not generate a valid native executable"
        );
    }
}
