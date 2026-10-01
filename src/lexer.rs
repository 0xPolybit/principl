use crate::diagnostics::Diagnostic;
use crate::source::{SourceFile, SourceLocation, SourceSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    IntegerLiteral(String),
    FloatingPointLiteral(String),
    StringLiteral(String),
    BooleanLiteral(bool),
    Keyword(Keyword),
    Operator(Operator),
    Punctuation(Punctuation),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Return,
    Let,
    Var,
    If,
    Else,
    While,
    For,
    In,
    Class,
    Struct,
    Init,
    SelfValue,
    Import,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    PlusAssign,
    MinusAssign,
    StarAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punctuation {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Arrow,
    Range,
}

pub fn lex(source: &SourceFile) -> Result<Vec<Token>, Diagnostic> {
    Lexer::new(source).lex()
}

struct Lexer<'a> {
    source: &'a SourceFile,
    offset: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a SourceFile) -> Self {
        Self {
            source,
            offset: 0,
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> Result<Vec<Token>, Diagnostic> {
        while let Some(character) = self.peek() {
            if character.is_whitespace() {
                self.advance();
            } else if character == '/' && self.peek_second() == Some('/') {
                self.skip_line_comment();
            } else if is_identifier_start(character) {
                self.scan_identifier();
            } else if character.is_ascii_digit() {
                self.scan_number()?;
            } else if character == '"' {
                self.scan_string()?;
            } else {
                self.scan_symbol()?;
            }
        }

        self.push(TokenKind::Eof, self.offset);
        Ok(self.tokens)
    }

    fn scan_identifier(&mut self) {
        let start = self.offset;
        self.advance();
        while self.peek().is_some_and(is_identifier_continue) {
            self.advance();
        }

        let text = &self.source.text()[start..self.offset];
        let kind = match text {
            "fn" => TokenKind::Keyword(Keyword::Fn),
            "return" => TokenKind::Keyword(Keyword::Return),
            "let" => TokenKind::Keyword(Keyword::Let),
            "var" => TokenKind::Keyword(Keyword::Var),
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "while" => TokenKind::Keyword(Keyword::While),
            "for" => TokenKind::Keyword(Keyword::For),
            "in" => TokenKind::Keyword(Keyword::In),
            "class" => TokenKind::Keyword(Keyword::Class),
            "struct" => TokenKind::Keyword(Keyword::Struct),
            "init" => TokenKind::Keyword(Keyword::Init),
            "self" => TokenKind::Keyword(Keyword::SelfValue),
            "import" => TokenKind::Keyword(Keyword::Import),
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            _ => TokenKind::Identifier(text.to_owned()),
        };

        self.push(kind, start);
    }

    fn scan_number(&mut self) -> Result<(), Diagnostic> {
        let start = self.offset;
        self.scan_digits();

        let mut is_float = false;
        if self.peek() == Some('.') && self.peek_second() != Some('.') {
            is_float = true;
            self.advance();
            self.scan_digits();
        }

        if matches!(self.peek(), Some('e' | 'E')) {
            is_float = true;
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }

            let exponent_start = self.offset;
            self.scan_digits();
            if self.offset == exponent_start {
                return Err(self.error("malformed floating-point literal", start, self.offset));
            }
        }

        let spelling = self.source.text()[start..self.offset].to_owned();
        let kind = if is_float {
            TokenKind::FloatingPointLiteral(spelling)
        } else {
            TokenKind::IntegerLiteral(spelling)
        };
        self.push(kind, start);
        Ok(())
    }

    fn scan_digits(&mut self) {
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_digit())
        {
            self.advance();
        }
    }

    fn scan_string(&mut self) -> Result<(), Diagnostic> {
        let start = self.offset;
        self.advance();
        let mut value = String::new();

        loop {
            match self.peek() {
                None | Some('\r' | '\n') => {
                    return Err(self.error("unterminated string literal", start, self.offset));
                }
                Some('"') => {
                    self.advance();
                    self.push(TokenKind::StringLiteral(value), start);
                    return Ok(());
                }
                Some('\\') => {
                    let escape_start = self.offset;
                    self.advance();
                    let escaped = match self.advance() {
                        Some(character) => character,
                        None => {
                            return Err(self.error(
                                "unterminated string literal",
                                start,
                                self.offset,
                            ));
                        }
                    };

                    match escaped {
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '0' => value.push('\0'),
                        '\r' | '\n' => {
                            return Err(self.error(
                                "unterminated string literal",
                                start,
                                self.offset,
                            ));
                        }
                        other => {
                            return Err(self.error(
                                format!("invalid escape sequence '\\{other}'"),
                                escape_start,
                                self.offset,
                            ));
                        }
                    }
                }
                Some(character) => {
                    self.advance();
                    value.push(character);
                }
            }
        }
    }

    fn scan_symbol(&mut self) -> Result<(), Diagnostic> {
        let start = self.offset;
        let Some(character) = self.advance() else {
            return Ok(());
        };
        let kind = match character {
            '+' if self.consume_if('=') => TokenKind::Operator(Operator::PlusAssign),
            '+' => TokenKind::Operator(Operator::Plus),
            '-' if self.consume_if('=') => TokenKind::Operator(Operator::MinusAssign),
            '-' if self.consume_if('>') => TokenKind::Punctuation(Punctuation::Arrow),
            '-' => TokenKind::Operator(Operator::Minus),
            '*' if self.consume_if('=') => TokenKind::Operator(Operator::StarAssign),
            '*' => TokenKind::Operator(Operator::Star),
            '/' => TokenKind::Operator(Operator::Slash),
            '%' => TokenKind::Operator(Operator::Percent),
            '=' if self.consume_if('=') => TokenKind::Operator(Operator::Equal),
            '=' => TokenKind::Operator(Operator::Assign),
            '!' if self.consume_if('=') => TokenKind::Operator(Operator::NotEqual),
            '!' => TokenKind::Operator(Operator::Not),
            '<' if self.consume_if('=') => TokenKind::Operator(Operator::LessEqual),
            '<' => TokenKind::Operator(Operator::Less),
            '>' if self.consume_if('=') => TokenKind::Operator(Operator::GreaterEqual),
            '>' => TokenKind::Operator(Operator::Greater),
            '&' if self.consume_if('&') => TokenKind::Operator(Operator::And),
            '|' if self.consume_if('|') => TokenKind::Operator(Operator::Or),
            '.' if self.consume_if('.') => TokenKind::Punctuation(Punctuation::Range),
            '.' => TokenKind::Punctuation(Punctuation::Dot),
            '(' => TokenKind::Punctuation(Punctuation::LeftParen),
            ')' => TokenKind::Punctuation(Punctuation::RightParen),
            '{' => TokenKind::Punctuation(Punctuation::LeftBrace),
            '}' => TokenKind::Punctuation(Punctuation::RightBrace),
            '[' => TokenKind::Punctuation(Punctuation::LeftBracket),
            ']' => TokenKind::Punctuation(Punctuation::RightBracket),
            ',' => TokenKind::Punctuation(Punctuation::Comma),
            ':' => TokenKind::Punctuation(Punctuation::Colon),
            ';' => TokenKind::Punctuation(Punctuation::Semicolon),
            _ => {
                return Err(self.error(
                    format!("unexpected character {character:?}"),
                    start,
                    self.offset,
                ));
            }
        };

        self.push(kind, start);
        Ok(())
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while !matches!(self.peek(), None | Some('\r' | '\n')) {
            self.advance();
        }
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.text()[self.offset..].chars().next()
    }

    fn peek_second(&self) -> Option<char> {
        let mut characters = self.source.text()[self.offset..].chars();
        characters.next()?;
        characters.next()
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.offset += character.len_utf8();
        Some(character)
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        let location = self.source.location(SourceSpan::new(start, self.offset));
        self.tokens.push(Token { kind, location });
    }

    fn error(&self, message: impl Into<String>, start: usize, end: usize) -> Diagnostic {
        Diagnostic::at(message, self.source.location(SourceSpan::new(start, end)))
    }
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{lex, Keyword, Operator, Punctuation, TokenKind};
    use crate::source::{SourceFile, SourceSpan};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("princi-lexer-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("temporary directory should be created");
            Self(path)
        }

        fn source(&self, name: &str, text: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, text).expect("source file should be written");
            path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn kinds(text: &str) -> Vec<TokenKind> {
        let source = SourceFile::from_text("test.prnc", text);
        lex(&source)
            .expect("source should lex")
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn lexes_a_valid_program_and_literal_kinds() {
        let tokens = kinds(
            "fn main() { let answer = 42; var ratio = 3.14; let avogadro = 6.02e23; let ok = true; return \"hello\\nworld\"; }",
        );

        assert!(tokens.contains(&TokenKind::IntegerLiteral("42".to_owned())));
        assert!(tokens.contains(&TokenKind::FloatingPointLiteral("3.14".to_owned())));
        assert!(tokens.contains(&TokenKind::FloatingPointLiteral("6.02e23".to_owned())));
        assert!(tokens.contains(&TokenKind::BooleanLiteral(true)));
        assert!(tokens.contains(&TokenKind::StringLiteral("hello\nworld".to_owned())));
        assert_eq!(tokens.last(), Some(&TokenKind::Eof));
    }

    #[test]
    fn recognizes_keywords_and_keeps_other_words_as_identifiers() {
        let tokens = kinds(
            "fn return let var if else while for in class struct init self import true false function",
        );

        assert_eq!(
            tokens,
            vec![
                TokenKind::Keyword(Keyword::Fn),
                TokenKind::Keyword(Keyword::Return),
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Keyword(Keyword::Var),
                TokenKind::Keyword(Keyword::If),
                TokenKind::Keyword(Keyword::Else),
                TokenKind::Keyword(Keyword::While),
                TokenKind::Keyword(Keyword::For),
                TokenKind::Keyword(Keyword::In),
                TokenKind::Keyword(Keyword::Class),
                TokenKind::Keyword(Keyword::Struct),
                TokenKind::Keyword(Keyword::Init),
                TokenKind::Keyword(Keyword::SelfValue),
                TokenKind::Keyword(Keyword::Import),
                TokenKind::BooleanLiteral(true),
                TokenKind::BooleanLiteral(false),
                TokenKind::Identifier("function".to_owned()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_all_requested_operators_and_punctuation() {
        let tokens =
            kinds("+ - * / % = == != < <= > >= && || ! += -= *= -> .. ( ) { } [ ] , . : ;");
        let mut expected = vec![
            TokenKind::Operator(Operator::Plus),
            TokenKind::Operator(Operator::Minus),
            TokenKind::Operator(Operator::Star),
            TokenKind::Operator(Operator::Slash),
            TokenKind::Operator(Operator::Percent),
            TokenKind::Operator(Operator::Assign),
            TokenKind::Operator(Operator::Equal),
            TokenKind::Operator(Operator::NotEqual),
            TokenKind::Operator(Operator::Less),
            TokenKind::Operator(Operator::LessEqual),
            TokenKind::Operator(Operator::Greater),
            TokenKind::Operator(Operator::GreaterEqual),
            TokenKind::Operator(Operator::And),
            TokenKind::Operator(Operator::Or),
            TokenKind::Operator(Operator::Not),
            TokenKind::Operator(Operator::PlusAssign),
            TokenKind::Operator(Operator::MinusAssign),
            TokenKind::Operator(Operator::StarAssign),
            TokenKind::Punctuation(Punctuation::Arrow),
            TokenKind::Punctuation(Punctuation::Range),
            TokenKind::Punctuation(Punctuation::LeftParen),
            TokenKind::Punctuation(Punctuation::RightParen),
            TokenKind::Punctuation(Punctuation::LeftBrace),
            TokenKind::Punctuation(Punctuation::RightBrace),
            TokenKind::Punctuation(Punctuation::LeftBracket),
            TokenKind::Punctuation(Punctuation::RightBracket),
            TokenKind::Punctuation(Punctuation::Comma),
            TokenKind::Punctuation(Punctuation::Dot),
            TokenKind::Punctuation(Punctuation::Colon),
            TokenKind::Punctuation(Punctuation::Semicolon),
        ];
        expected.push(TokenKind::Eof);

        assert_eq!(tokens, expected);
    }

    #[test]
    fn skips_comments_and_whitespace_but_not_comment_markers_in_strings() {
        let tokens = kinds("let x = 1; // trailing comment\r\n// full line\nx = \"// text\";");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Identifier("x".to_owned()),
                TokenKind::Operator(Operator::Assign),
                TokenKind::IntegerLiteral("1".to_owned()),
                TokenKind::Punctuation(Punctuation::Semicolon),
                TokenKind::Identifier("x".to_owned()),
                TokenKind::Operator(Operator::Assign),
                TokenKind::StringLiteral("// text".to_owned()),
                TokenKind::Punctuation(Punctuation::Semicolon),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn records_file_line_column_and_byte_spans_on_tokens() {
        let source = SourceFile::from_text("hello.prnc", "// π\r\nlet café = 2");
        let tokens = lex(&source).expect("source should lex");
        let identifier = tokens
            .iter()
            .find(|token| token.kind == TokenKind::Identifier("café".to_owned()))
            .expect("identifier token should exist");

        assert_eq!(identifier.location.file, PathBuf::from("hello.prnc"));
        assert_eq!(
            (identifier.location.line, identifier.location.column),
            (2, 5)
        );
        assert_eq!(identifier.location.span, SourceSpan::new(11, 16));
        assert_eq!(
            source
                .text()
                .get(identifier.location.span.start..identifier.location.span.end),
            Some("café")
        );
    }

    #[test]
    fn reports_unterminated_strings_at_the_opening_quote() {
        let source = SourceFile::from_text("hello.prnc", "first\n           \"unfinished");
        let error = lex(&source).expect_err("unterminated string should fail");
        let location = error.location().expect("lexical error should be located");

        assert_eq!((location.line, location.column), (2, 12));
        assert_eq!(location.span, SourceSpan::new(17, 28));
        assert_eq!(
            error.to_string(),
            "hello.prnc:2:12: error: unterminated string literal"
        );
    }

    #[test]
    fn malformed_literals_and_unexpected_characters_are_diagnostics() {
        for (text, message) in [
            ("\"line\nbreak\"", "unterminated string literal"),
            ("\"bad\\q\"", "invalid escape sequence"),
            ("12e+", "malformed floating-point literal"),
            ("@", "unexpected character"),
        ] {
            let source = SourceFile::from_text("bad.princi", text);
            let error = lex(&source).expect_err("malformed input should not lex");
            assert!(error.message().contains(message), "{error}");
            assert!(error.location().is_some(), "{error}");
        }
    }

    #[test]
    fn both_source_extensions_load_and_lex_the_same_program() {
        let dir = TestDir::new();
        let text = "fn main() { return true; }\n";
        let mut extension_tokens = Vec::new();

        for name in ["hello.prnc", "hello.princi"] {
            let path = dir.source(name, text);
            let source = SourceFile::load(&path).expect("source file should load");
            let tokens = lex(&source).expect("source should lex identically");
            assert_eq!(tokens[0].location.file, path);
            extension_tokens.push(
                tokens
                    .into_iter()
                    .map(|token| token.kind)
                    .collect::<Vec<_>>(),
            );
        }

        assert_eq!(extension_tokens[0], extension_tokens[1]);
    }
}
