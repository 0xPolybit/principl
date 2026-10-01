use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    message: String,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
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
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "princi: error: {}", self.message)
    }
}

impl std::error::Error for Diagnostic {}
