use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{Diagnostic, DiagnosticCode};

/// A byte range in a source file. The start is inclusive and the end is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A source position suitable for tokens and user-facing diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub span: SourceSpan,
}

/// Loaded UTF-8 source plus the line index used to map byte offsets to locations.
#[derive(Debug, Clone)]
pub struct SourceFile {
    path: PathBuf,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Diagnostic> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|error| {
            let message = if error.kind() == std::io::ErrorKind::NotFound {
                format!("source file '{}' was not found", path.display())
            } else {
                format!("could not read source file '{}': {error}", path.display())
            };
            Diagnostic::coded(DiagnosticCode::SourceFile, message)
        })?;

        Ok(Self::from_text(path.to_path_buf(), text))
    }

    pub fn from_text(path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        let text = text.into();
        let line_starts = Self::index_lines(&text);

        Self {
            path: path.into(),
            text,
            line_starts,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn location(&self, span: SourceSpan) -> SourceLocation {
        let offset = self.char_boundary_at_or_before(span.start.min(self.text.len()));
        let line_index = self
            .line_starts
            .partition_point(|line_start| *line_start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line_index];
        let column = self.text[line_start..offset].chars().count() + 1;

        SourceLocation {
            file: self.path.clone(),
            line: line_index + 1,
            column,
            span,
        }
    }

    pub(crate) fn excerpt(&self, span: SourceSpan) -> (String, usize, usize) {
        let location = self.location(span);
        let line_index = location.line.saturating_sub(1);
        let line_start = self.line_starts[line_index];
        let mut line_end = self
            .line_starts
            .get(line_index + 1)
            .copied()
            .unwrap_or(self.text.len());
        while line_end > line_start && matches!(self.text.as_bytes()[line_end - 1], b'\r' | b'\n') {
            line_end -= 1;
        }

        let start = self
            .char_boundary_at_or_before(span.start.min(self.text.len()))
            .clamp(line_start, line_end);
        let end = self
            .char_boundary_at_or_before(span.end.min(self.text.len()))
            .clamp(start, line_end);
        let line = &self.text[line_start..line_end];
        let before = &self.text[line_start..start];
        let highlighted = &self.text[start..end];
        let column = visual_width(before, 0);
        let width = visual_width(highlighted, column)
            .saturating_sub(column)
            .max(1);
        let rendered_line = expand_tabs(line);
        (rendered_line, column, width)
    }

    fn char_boundary_at_or_before(&self, mut offset: usize) -> usize {
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn index_lines(text: &str) -> Vec<usize> {
        let bytes = text.as_bytes();
        let mut starts = vec![0];
        let mut offset = 0;

        while offset < bytes.len() {
            match bytes[offset] {
                b'\r' if bytes.get(offset + 1) == Some(&b'\n') => {
                    offset += 2;
                    starts.push(offset);
                }
                b'\r' | b'\n' => {
                    offset += 1;
                    starts.push(offset);
                }
                _ => offset += 1,
            }
        }

        starts
    }
}

fn visual_width(text: &str, mut column: usize) -> usize {
    for character in text.chars() {
        if character == '\t' {
            column += 4 - (column % 4);
        } else {
            column += 1;
        }
    }
    column
}

fn expand_tabs(text: &str) -> String {
    let mut expanded = String::new();
    let mut column = 0;
    for character in text.chars() {
        if character == '\t' {
            let spaces = 4 - (column % 4);
            expanded.push_str(&" ".repeat(spaces));
            column += spaces;
        } else {
            expanded.push(character);
            column += 1;
        }
    }
    expanded
}

#[cfg(test)]
mod tests {
    use super::{SourceFile, SourceSpan};
    use std::path::PathBuf;

    #[test]
    fn maps_utf8_byte_spans_to_one_based_character_columns() {
        let source = SourceFile::from_text("hello.prnc", "a\r\néx");
        let location = source.location(SourceSpan::new(3, 5));

        assert_eq!(location.file, PathBuf::from("hello.prnc"));
        assert_eq!(location.line, 2);
        assert_eq!(location.column, 1);
        assert_eq!(location.span, SourceSpan::new(3, 5));
        assert_eq!(source.location(SourceSpan::new(5, 6)).column, 2);
        assert_eq!(source.location(SourceSpan::new(6, 6)).column, 3);
    }

    #[test]
    fn handles_lone_carriage_return_and_end_of_file_after_newline() {
        let source = SourceFile::from_text("hello.prnc", "a\rb\n");

        assert_eq!(source.location(SourceSpan::new(2, 3)).line, 2);
        let eof = source.location(SourceSpan::new(4, 4));
        assert_eq!((eof.line, eof.column), (3, 1));
    }
}
