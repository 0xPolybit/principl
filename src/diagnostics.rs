use std::fmt;

use crate::source::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    message: String,
    location: Option<SourceLocation>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }

    pub fn at(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            message: message.into(),
            location: Some(location),
        }
    }

    pub fn unsupported_extension(extension: &str) -> Self {
        Self::new(format!(
            "unsupported source extension '{extension}'; expected .prnc or .princi"
        ))
    }

    pub fn pipeline_stage_incomplete(stage: &str) -> Self {
        Self::new(format!(
            "compiler pipeline is incomplete: the {stage} stage is not implemented yet"
        ))
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
        if let Some(location) = &self.location {
            write!(
                f,
                "{}:{}:{}: error: {}",
                location.file.display(),
                location.line,
                location.column,
                self.message
            )
        } else {
            write!(f, "princi: error: {}", self.message)
        }
    }
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
    use super::Diagnostic;
    use crate::source::{SourceFile, SourceSpan};

    #[test]
    fn formats_located_diagnostics_and_retains_the_source_span() {
        let source = SourceFile::from_text("hello.prnc", "line one\n           \"unfinished");
        let span = SourceSpan::new(20, 30);
        let error = Diagnostic::at("unterminated string literal", source.location(span));

        assert_eq!(
            error.to_string(),
            "hello.prnc:2:12: error: unterminated string literal"
        );
        assert_eq!(
            error.location().expect("location should be retained").span,
            span
        );
    }

    #[test]
    fn formats_non_source_diagnostics() {
        assert_eq!(
            Diagnostic::new("missing input").to_string(),
            "princi: error: missing input"
        );
    }
}
