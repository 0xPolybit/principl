use crate::ast::*;
use crate::diagnostics::Diagnostic;
use crate::lexer::{Keyword, Operator, Punctuation, Token, TokenKind};
use crate::source::{SourceFile, SourceSpan};

const RANGE_PRECEDENCE: u8 = 1;
const PREFIX_PRECEDENCE: u8 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: &SourceFile, mut tokens: Vec<Token>) -> ParseResult {
    let eof_span = SourceSpan::new(source.text().len(), source.text().len());
    let eof = Token {
        kind: TokenKind::Eof,
        location: source.location(eof_span),
    };
    if !tokens
        .last()
        .is_some_and(|token| token.kind == TokenKind::Eof)
    {
        tokens.push(eof.clone());
    }

    Parser {
        source,
        tokens,
        eof,
        cursor: 0,
        diagnostics: Vec::new(),
    }
    .parse_program()
}

struct Parser<'a> {
    source: &'a SourceFile,
    tokens: Vec<Token>,
    eof: Token,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser<'_> {
    fn parse_program(mut self) -> ParseResult {
        let mut declarations = Vec::new();

        while !self.at_eof() {
            let before = self.cursor;
            match self.parse_declaration() {
                Ok(declaration) => declarations.push(declaration),
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.recover_top_level(before);
                }
            }

            if self.cursor == before && !self.at_eof() {
                self.advance();
            }
        }

        ParseResult {
            program: Program {
                declarations,
                span: SourceSpan::new(0, self.source.text().len()),
            },
            diagnostics: self.diagnostics,
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, Diagnostic> {
        match self.current().kind {
            TokenKind::Keyword(Keyword::Fn) => self.parse_function().map(Declaration::Function),
            TokenKind::Keyword(Keyword::Class) => self
                .parse_type_declaration(TypeDeclarationKind::Class)
                .map(Declaration::Class),
            TokenKind::Keyword(Keyword::Struct) => self
                .parse_type_declaration(TypeDeclarationKind::Struct)
                .map(Declaration::Struct),
            TokenKind::Keyword(Keyword::Import) => self.parse_import().map(Declaration::Import),
            _ => Err(self.error_here("expected a function, class, struct, or import declaration")),
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected function name")?;
        let parameters = self.parse_parameters()?;
        let return_type = if self.eat_punctuation(Punctuation::Arrow) {
            Some(self.parse_type_reference("expected return type after '->'")?)
        } else {
            None
        };
        let body = self.parse_block()?;
        let span = SourceSpan::new(start, body.span.end);

        Ok(FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
            span,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, Diagnostic> {
        self.expect_punctuation(Punctuation::LeftParen, "expected '(' before parameters")?;
        let mut parameters = Vec::new();

        if !self.check_punctuation(Punctuation::RightParen) {
            loop {
                let start = self.current().location.span.start;
                let name = self.expect_identifier("expected parameter name")?;
                self.expect_punctuation(Punctuation::Colon, "expected ':' after parameter name")?;
                let type_reference = self.parse_type_reference("expected parameter type")?;
                parameters.push(Parameter {
                    name,
                    span: SourceSpan::new(start, type_reference.span.end),
                    type_reference,
                });

                if !self.eat_punctuation(Punctuation::Comma) {
                    break;
                }
                if self.check_punctuation(Punctuation::RightParen) {
                    break;
                }
            }
        }

        self.expect_punctuation(Punctuation::RightParen, "expected ')' after parameters")?;
        Ok(parameters)
    }

    fn parse_type_reference(&mut self, message: &str) -> Result<TypeReference, Diagnostic> {
        let name = self.expect_identifier(message)?;
        let mut arguments = Vec::new();
        let mut end = name.span.end;
        if self.eat_operator(Operator::Less) {
            loop {
                arguments.push(self.parse_type_reference("expected type argument")?);
                if self.eat_punctuation(Punctuation::Comma) {
                    continue;
                }
                let close = self.current().clone();
                if close.kind != TokenKind::Operator(Operator::Greater) {
                    return Err(Diagnostic::at(
                        "expected '>' after type arguments",
                        close.location,
                    ));
                }
                self.advance();
                end = close.location.span.end;
                break;
            }
        }
        Ok(TypeReference {
            name: name.name,
            arguments,
            span: SourceSpan::new(name.span.start, end),
        })
    }

    fn parse_type_declaration(
        &mut self,
        kind: TypeDeclarationKind,
    ) -> Result<TypeDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected class or struct name")?;
        self.expect_punctuation(Punctuation::LeftBrace, "expected '{' after type name")?;
        let mut members = Vec::new();

        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let before = self.cursor;
            match self.parse_class_member() {
                Ok(member) => members.push(member),
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.recover_class_member(before);
                }
            }

            if self.cursor == before && !self.at_eof() {
                self.advance();
            }
        }

        let close =
            self.expect_punctuation(Punctuation::RightBrace, "expected '}' to close type")?;
        Ok(TypeDeclaration {
            kind,
            name,
            members,
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_class_member(&mut self) -> Result<ClassMember, Diagnostic> {
        match self.current().kind {
            TokenKind::Keyword(Keyword::Fn) => self.parse_function().map(ClassMember::Method),
            TokenKind::Keyword(Keyword::Init) => {
                self.parse_initializer().map(ClassMember::Initializer)
            }
            TokenKind::Identifier(_) => self.parse_field().map(ClassMember::Field),
            _ => Err(self.error_here("expected a field, method, or init constructor")),
        }
    }

    fn parse_field(&mut self) -> Result<FieldDeclaration, Diagnostic> {
        let name = self.expect_identifier("expected field name")?;
        let start = name.span.start;
        self.expect_punctuation(Punctuation::Colon, "expected ':' after field name")?;
        let type_reference = self.parse_type_reference("expected field type")?;
        let span = SourceSpan::new(start, type_reference.span.end);
        self.finish_member()?;

        Ok(FieldDeclaration {
            name,
            type_reference,
            span,
        })
    }

    fn parse_initializer(&mut self) -> Result<InitializerDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let parameters = self.parse_parameters()?;
        let body = self.parse_block()?;
        Ok(InitializerDeclaration {
            parameters,
            span: SourceSpan::new(start, body.span.end),
            body,
        })
    }

    fn parse_import(&mut self) -> Result<ImportDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let mut path = vec![self.expect_identifier("expected import path")?];
        while self.eat_punctuation(Punctuation::Dot) {
            path.push(self.expect_identifier("expected name after '.' in import path")?);
        }
        let end = path.last().map_or(start, |segment| segment.span.end);
        self.finish_statement()?;
        Ok(ImportDeclaration {
            path,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_block(&mut self) -> Result<Block, Diagnostic> {
        let open =
            self.expect_punctuation(Punctuation::LeftBrace, "expected '{' to start block")?;
        let mut statements = Vec::new();

        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let before = self.cursor;
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.recover_statement(before);
                }
            }

            if self.cursor == before && !self.at_eof() {
                self.advance();
            }
        }

        let close =
            self.expect_punctuation(Punctuation::RightBrace, "expected '}' to close block")?;
        Ok(Block {
            statements,
            span: SourceSpan::new(open.location.span.start, close.location.span.end),
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, Diagnostic> {
        match self.current().kind {
            TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Var) => {
                self.parse_variable_statement()
            }
            TokenKind::Keyword(Keyword::If) => self.parse_if_statement(),
            TokenKind::Keyword(Keyword::While) => self.parse_while_statement(),
            TokenKind::Keyword(Keyword::For) => self.parse_for_statement(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_statement(),
            TokenKind::Punctuation(Punctuation::LeftBrace) => {
                let block = self.parse_block()?;
                self.finish_statement()?;
                Ok(Statement {
                    span: block.span,
                    kind: StatementKind::Block(block),
                })
            }
            _ => self.parse_expression_or_assignment_statement(),
        }
    }

    fn parse_variable_statement(&mut self) -> Result<Statement, Diagnostic> {
        let keyword = self.advance();
        let mutable = keyword.kind == TokenKind::Keyword(Keyword::Var);
        let name = self.expect_identifier("expected variable name")?;
        let type_reference = if self.eat_punctuation(Punctuation::Colon) {
            Some(self.parse_type_reference("expected type after ':'")?)
        } else {
            None
        };
        let initializer = if self.eat_operator(Operator::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if type_reference.is_none() && initializer.is_none() {
            return Err(Diagnostic::at(
                "variable declaration needs a type or an initializer",
                self.source.location(name.span),
            ));
        }

        let end = initializer
            .as_ref()
            .map(|expression| expression.span.end)
            .or_else(|| type_reference.as_ref().map(|ty| ty.span.end))
            .unwrap_or(name.span.end);
        self.finish_statement()?;

        Ok(Statement {
            kind: StatementKind::Variable(VariableDeclaration {
                mutable,
                name,
                type_reference,
                initializer,
            }),
            span: SourceSpan::new(keyword.location.span.start, end),
        })
    }

    fn parse_expression_or_assignment_statement(&mut self) -> Result<Statement, Diagnostic> {
        let target_or_expression = self.parse_expression()?;
        if let Some(operator) = self.assignment_operator() {
            if !is_assignment_target(&target_or_expression) {
                return Err(Diagnostic::at(
                    "invalid assignment target",
                    self.source.location(target_or_expression.span),
                ));
            }

            self.advance();
            let value = self.parse_expression()?;
            let span = SourceSpan::new(target_or_expression.span.start, value.span.end);
            self.finish_statement()?;
            Ok(Statement {
                kind: StatementKind::Assignment(Assignment {
                    target: target_or_expression,
                    operator,
                    value,
                }),
                span,
            })
        } else {
            let span = target_or_expression.span;
            self.finish_statement()?;
            Ok(Statement {
                kind: StatementKind::Expression(target_or_expression),
                span,
            })
        }
    }

    fn parse_if_statement(&mut self) -> Result<Statement, Diagnostic> {
        let start = self.advance().location.span.start;
        let condition = self.parse_expression_bp(RANGE_PRECEDENCE, false)?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.eat_keyword(Keyword::Else) {
            Some(Box::new(if self.check_keyword(Keyword::If) {
                self.parse_if_statement()?
            } else {
                let block = self.parse_block()?;
                Statement {
                    span: block.span,
                    kind: StatementKind::Block(block),
                }
            }))
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map_or(then_branch.span.end, |statement| statement.span.end);
        self.finish_statement()?;

        Ok(Statement {
            kind: StatementKind::If(IfStatement {
                condition,
                then_branch,
                else_branch,
            }),
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, Diagnostic> {
        let start = self.advance().location.span.start;
        let condition = self.parse_expression_bp(RANGE_PRECEDENCE, false)?;
        let body = self.parse_block()?;
        self.finish_statement()?;
        Ok(Statement {
            span: SourceSpan::new(start, body.span.end),
            kind: StatementKind::While(WhileStatement { condition, body }),
        })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, Diagnostic> {
        let start = self.advance().location.span.start;
        let variable = self.expect_identifier("expected loop variable after 'for'")?;
        self.expect_keyword(Keyword::In, "expected 'in' after loop variable")?;
        let range = self.parse_expression_bp(RANGE_PRECEDENCE, false)?;
        if !is_range_expression(&range) {
            return Err(Diagnostic::at(
                "expected a range expression after 'in'",
                self.source.location(range.span),
            ));
        }
        let body = self.parse_block()?;
        self.finish_statement()?;
        Ok(Statement {
            span: SourceSpan::new(start, body.span.end),
            kind: StatementKind::For(ForStatement {
                variable,
                range,
                body,
            }),
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, Diagnostic> {
        let keyword = self.advance();
        let value = if self.check_punctuation(Punctuation::Semicolon)
            || self.check_punctuation(Punctuation::RightBrace)
            || self.at_eof()
            || self.line_break_before_current()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };
        let end = value
            .as_ref()
            .map_or(keyword.location.span.end, |expression| expression.span.end);
        self.finish_statement()?;
        Ok(Statement {
            kind: StatementKind::Return(value),
            span: SourceSpan::new(keyword.location.span.start, end),
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, Diagnostic> {
        self.parse_expression_bp(RANGE_PRECEDENCE, true)
    }

    fn parse_expression_bp(
        &mut self,
        min_precedence: u8,
        allow_construction: bool,
    ) -> Result<Expression, Diagnostic> {
        let mut left = self.parse_prefix_expression(allow_construction)?;

        loop {
            if self.check_punctuation(Punctuation::LeftParen) {
                left = self.parse_call_expression(left)?;
                continue;
            }
            if self.eat_punctuation(Punctuation::Dot) {
                let member = self.expect_identifier("expected member name after '.'")?;
                let span = SourceSpan::new(left.span.start, member.span.end);
                left = Expression {
                    kind: ExpressionKind::Member {
                        object: Box::new(left),
                        member,
                    },
                    span,
                };
                continue;
            }
            if self.check_punctuation(Punctuation::LeftBracket) {
                self.advance();
                let index = self.parse_expression()?;
                let close = self.expect_punctuation(
                    Punctuation::RightBracket,
                    "expected ']' after index expression",
                )?;
                left = Expression {
                    span: SourceSpan::new(left.span.start, close.location.span.end),
                    kind: ExpressionKind::Index {
                        object: Box::new(left),
                        index: Box::new(index),
                    },
                };
                continue;
            }

            if self.check_punctuation(Punctuation::Range) && min_precedence <= RANGE_PRECEDENCE {
                if matches!(&left.kind, ExpressionKind::Range { .. }) {
                    return Err(self.error_here(
                        "range operator '..' is non-associative; parenthesize the range",
                    ));
                }
                self.advance();
                let right = self.parse_expression_bp(RANGE_PRECEDENCE + 1, allow_construction)?;
                let span = SourceSpan::new(left.span.start, right.span.end);
                left = Expression {
                    kind: ExpressionKind::Range {
                        start: Box::new(left),
                        end: Box::new(right),
                    },
                    span,
                };
                continue;
            }

            let Some((precedence, operator)) = self.binary_operator() else {
                break;
            };
            if precedence < min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_expression_bp(precedence + 1, allow_construction)?;
            let span = SourceSpan::new(left.span.start, right.span.end);
            left = Expression {
                kind: ExpressionKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_prefix_expression(
        &mut self,
        allow_construction: bool,
    ) -> Result<Expression, Diagnostic> {
        let operator = match self.current().kind {
            TokenKind::Operator(Operator::Plus) => Some(UnaryOperator::Positive),
            TokenKind::Operator(Operator::Minus) => Some(UnaryOperator::Negative),
            TokenKind::Operator(Operator::Not) => Some(UnaryOperator::Not),
            _ => None,
        };

        if let Some(operator) = operator {
            let start = self.advance().location.span.start;
            let operand = self.parse_expression_bp(PREFIX_PRECEDENCE, allow_construction)?;
            return Ok(Expression {
                span: SourceSpan::new(start, operand.span.end),
                kind: ExpressionKind::Unary {
                    operator,
                    operand: Box::new(operand),
                },
            });
        }

        self.parse_primary_expression(allow_construction)
    }

    fn parse_primary_expression(
        &mut self,
        allow_construction: bool,
    ) -> Result<Expression, Diagnostic> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::Identifier(name) => {
                self.advance();
                let identifier = Identifier {
                    name,
                    span: token.location.span,
                };
                if allow_construction && self.check_punctuation(Punctuation::LeftBrace) {
                    self.parse_construction_expression(identifier)
                } else {
                    Ok(Expression {
                        span: token.location.span,
                        kind: ExpressionKind::Identifier(identifier),
                    })
                }
            }
            TokenKind::Keyword(Keyword::SelfValue) => {
                self.advance();
                Ok(Expression {
                    span: token.location.span,
                    kind: ExpressionKind::SelfValue,
                })
            }
            TokenKind::IntegerLiteral(value) => {
                self.advance();
                Ok(Expression {
                    span: token.location.span,
                    kind: ExpressionKind::Literal(Literal::Integer(value)),
                })
            }
            TokenKind::FloatingPointLiteral(value) => {
                self.advance();
                Ok(Expression {
                    span: token.location.span,
                    kind: ExpressionKind::Literal(Literal::FloatingPoint(value)),
                })
            }
            TokenKind::StringLiteral(value) => {
                self.advance();
                Ok(Expression {
                    span: token.location.span,
                    kind: ExpressionKind::Literal(Literal::String(value)),
                })
            }
            TokenKind::BooleanLiteral(value) => {
                self.advance();
                Ok(Expression {
                    span: token.location.span,
                    kind: ExpressionKind::Literal(Literal::Boolean(value)),
                })
            }
            TokenKind::Punctuation(Punctuation::LeftParen) => {
                let open = self.advance();
                let expression = self.parse_expression()?;
                let close = self
                    .expect_punctuation(Punctuation::RightParen, "expected ')' after expression")?;
                Ok(Expression {
                    kind: ExpressionKind::Group(Box::new(expression)),
                    span: SourceSpan::new(open.location.span.start, close.location.span.end),
                })
            }
            TokenKind::Punctuation(Punctuation::LeftBracket) => self.parse_list_expression(),
            _ => Err(self.error_here("expected an expression")),
        }
    }

    fn parse_list_expression(&mut self) -> Result<Expression, Diagnostic> {
        let open = self.advance();
        let mut elements = Vec::new();

        if !self.check_punctuation(Punctuation::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.eat_punctuation(Punctuation::Comma) {
                    break;
                }
                if self.check_punctuation(Punctuation::RightBracket) {
                    break;
                }
            }
        }

        let close = self.expect_punctuation(
            Punctuation::RightBracket,
            "expected ']' after list elements",
        )?;
        Ok(Expression {
            kind: ExpressionKind::List(elements),
            span: SourceSpan::new(open.location.span.start, close.location.span.end),
        })
    }

    fn parse_construction_expression(
        &mut self,
        type_name: Identifier,
    ) -> Result<Expression, Diagnostic> {
        self.advance();
        let start = type_name.span.start;
        let mut fields = Vec::new();

        if !self.check_punctuation(Punctuation::RightBrace) {
            loop {
                let name = self.expect_identifier("expected field name in construction")?;
                let start = name.span.start;
                self.expect_punctuation(
                    Punctuation::Colon,
                    "expected ':' after constructed field name",
                )?;
                let value = self.parse_expression()?;
                fields.push(FieldInitializer {
                    span: SourceSpan::new(start, value.span.end),
                    name,
                    value,
                });

                if !self.eat_punctuation(Punctuation::Comma) {
                    break;
                }
                if self.check_punctuation(Punctuation::RightBrace) {
                    break;
                }
            }
        }

        let close = self.expect_punctuation(
            Punctuation::RightBrace,
            "expected '}' after constructed fields",
        )?;
        Ok(Expression {
            kind: ExpressionKind::Construction { type_name, fields },
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_call_expression(&mut self, callee: Expression) -> Result<Expression, Diagnostic> {
        self.advance();
        let mut arguments = Vec::new();

        if !self.check_punctuation(Punctuation::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.eat_punctuation(Punctuation::Comma) {
                    break;
                }
                if self.check_punctuation(Punctuation::RightParen) {
                    break;
                }
            }
        }

        let close =
            self.expect_punctuation(Punctuation::RightParen, "expected ')' after arguments")?;
        Ok(Expression {
            span: SourceSpan::new(callee.span.start, close.location.span.end),
            kind: ExpressionKind::Call {
                callee: Box::new(callee),
                arguments,
            },
        })
    }

    fn binary_operator(&self) -> Option<(u8, BinaryOperator)> {
        let TokenKind::Operator(operator) = self.current().kind else {
            return None;
        };

        Some(match operator {
            Operator::Or => (2, BinaryOperator::Or),
            Operator::And => (3, BinaryOperator::And),
            Operator::Equal => (4, BinaryOperator::Equal),
            Operator::NotEqual => (4, BinaryOperator::NotEqual),
            Operator::Less => (5, BinaryOperator::Less),
            Operator::LessEqual => (5, BinaryOperator::LessEqual),
            Operator::Greater => (5, BinaryOperator::Greater),
            Operator::GreaterEqual => (5, BinaryOperator::GreaterEqual),
            Operator::Plus => (6, BinaryOperator::Add),
            Operator::Minus => (6, BinaryOperator::Subtract),
            Operator::Star => (7, BinaryOperator::Multiply),
            Operator::Slash => (7, BinaryOperator::Divide),
            Operator::Percent => (7, BinaryOperator::Remainder),
            _ => return None,
        })
    }

    fn assignment_operator(&self) -> Option<AssignmentOperator> {
        match self.current().kind {
            TokenKind::Operator(Operator::Assign) => Some(AssignmentOperator::Assign),
            TokenKind::Operator(Operator::PlusAssign) => Some(AssignmentOperator::AddAssign),
            TokenKind::Operator(Operator::MinusAssign) => Some(AssignmentOperator::SubtractAssign),
            TokenKind::Operator(Operator::StarAssign) => Some(AssignmentOperator::MultiplyAssign),
            _ => None,
        }
    }

    fn finish_statement(&mut self) -> Result<(), Diagnostic> {
        if self.eat_punctuation(Punctuation::Semicolon)
            || self.check_punctuation(Punctuation::RightBrace)
            || self.at_eof()
            || self.line_break_before_current()
        {
            Ok(())
        } else {
            Err(self.error_here("expected ';' or a line break after statement"))
        }
    }

    fn finish_member(&mut self) -> Result<(), Diagnostic> {
        if self.eat_punctuation(Punctuation::Semicolon)
            || self.eat_punctuation(Punctuation::Comma)
            || self.check_punctuation(Punctuation::RightBrace)
            || self.at_eof()
            || self.line_break_before_current()
        {
            Ok(())
        } else {
            Err(self.error_here("expected a line break or separator after field"))
        }
    }

    fn line_break_before_current(&self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.current().location.line > self.tokens[self.cursor - 1].location.line
    }

    fn recover_top_level(&mut self, before: usize) {
        if self.cursor == before && !self.at_eof() {
            self.advance();
        }
        while !self.at_eof() {
            if self.is_declaration_start() {
                return;
            }
            if self.eat_punctuation(Punctuation::Semicolon) {
                return;
            }
            self.advance();
        }
    }

    fn recover_statement(&mut self, before: usize) {
        let mut advanced = self.cursor > before;
        if !advanced && !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            self.advance();
            advanced = true;
        }

        while !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            if self.eat_punctuation(Punctuation::Semicolon) {
                return;
            }
            if advanced && self.line_break_before_current() && self.can_start_statement() {
                return;
            }
            self.advance();
            advanced = true;
        }
    }

    fn recover_class_member(&mut self, before: usize) {
        let mut advanced = self.cursor > before;
        if !advanced && !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            self.advance();
            advanced = true;
        }

        while !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            if self.eat_punctuation(Punctuation::Semicolon)
                || self.eat_punctuation(Punctuation::Comma)
            {
                return;
            }
            if advanced && self.line_break_before_current() && self.can_start_class_member() {
                return;
            }
            self.advance();
            advanced = true;
        }
    }

    fn is_declaration_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Keyword(Keyword::Fn | Keyword::Class | Keyword::Struct | Keyword::Import)
        )
    }

    fn can_start_class_member(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Fn | Keyword::Init)
        )
    }

    fn can_start_statement(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Identifier(_)
                | TokenKind::IntegerLiteral(_)
                | TokenKind::FloatingPointLiteral(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::BooleanLiteral(_)
                | TokenKind::Keyword(
                    Keyword::Let
                        | Keyword::Var
                        | Keyword::If
                        | Keyword::While
                        | Keyword::For
                        | Keyword::Return
                        | Keyword::SelfValue
                )
                | TokenKind::Operator(Operator::Plus | Operator::Minus | Operator::Not)
                | TokenKind::Punctuation(
                    Punctuation::LeftBrace | Punctuation::LeftParen | Punctuation::LeftBracket
                )
        )
    }

    fn expect_identifier(&mut self, message: &str) -> Result<Identifier, Diagnostic> {
        let token = self.current().clone();
        if let TokenKind::Identifier(name) = token.kind {
            self.advance();
            Ok(Identifier {
                name,
                span: token.location.span,
            })
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword, message: &str) -> Result<Token, Diagnostic> {
        if self.check_keyword(keyword) {
            Ok(self.advance())
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_punctuation(
        &mut self,
        punctuation: Punctuation,
        message: &str,
    ) -> Result<Token, Diagnostic> {
        if self.check_punctuation(punctuation) {
            Ok(self.advance())
        } else {
            Err(self.error_here(message))
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if self.check_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_keyword(&self, keyword: Keyword) -> bool {
        self.current().kind == TokenKind::Keyword(keyword)
    }

    fn eat_punctuation(&mut self, punctuation: Punctuation) -> bool {
        if self.check_punctuation(punctuation) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_punctuation(&self, punctuation: Punctuation) -> bool {
        self.current().kind == TokenKind::Punctuation(punctuation)
    }

    fn eat_operator(&mut self, operator: Operator) -> bool {
        if self.current().kind == TokenKind::Operator(operator) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.cursor).unwrap_or(&self.eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if token.kind != TokenKind::Eof {
            self.cursor = self.cursor.saturating_add(1);
        }
        token
    }

    fn at_eof(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    fn error_here(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::at(message, self.current().location.clone())
    }
}

fn is_assignment_target(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Identifier(_)
        | ExpressionKind::Member { .. }
        | ExpressionKind::Index { .. } => true,
        ExpressionKind::Group(expression) => is_assignment_target(expression),
        _ => false,
    }
}

fn is_range_expression(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Range { .. } => true,
        ExpressionKind::Group(expression) => is_range_expression(expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::ast::{
        BinaryOperator, ClassMember, Declaration, Expression, ExpressionKind, Literal, Statement,
        StatementKind, TypeDeclarationKind,
    };
    use crate::source::SourceFile;

    fn parse_text(path: &str, text: &str) -> super::ParseResult {
        let source = SourceFile::from_text(path, text);
        let tokens = crate::lexer::lex(&source).expect("test source should lex");
        parse(&source, tokens)
    }

    fn only_function(result: &super::ParseResult) -> &crate::ast::FunctionDeclaration {
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let [Declaration::Function(function)] = result.program.declarations.as_slice() else {
            panic!("expected exactly one function declaration");
        };
        function
    }

    fn only_statement(function: &crate::ast::FunctionDeclaration) -> &Statement {
        let [statement] = function.body.statements.as_slice() else {
            panic!("expected exactly one statement");
        };
        statement
    }

    fn type_reference_snapshot(type_reference: &crate::ast::TypeReference) -> String {
        if type_reference.arguments.is_empty() {
            type_reference.name.clone()
        } else {
            format!(
                "{}<{}>",
                type_reference.name,
                type_reference
                    .arguments
                    .iter()
                    .map(type_reference_snapshot)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    fn expression_snapshot(expression: &Expression) -> String {
        match &expression.kind {
            ExpressionKind::Identifier(identifier) => identifier.name.clone(),
            ExpressionKind::SelfValue => "self".to_owned(),
            ExpressionKind::Literal(Literal::Integer(value))
            | ExpressionKind::Literal(Literal::FloatingPoint(value)) => value.clone(),
            ExpressionKind::Literal(Literal::String(value)) => format!("{value:?}"),
            ExpressionKind::Literal(Literal::Boolean(value)) => value.to_string(),
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let operator = match operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Remainder => "%",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };
                format!(
                    "({} {operator} {})",
                    expression_snapshot(left),
                    expression_snapshot(right)
                )
            }
            ExpressionKind::Group(inner) => format!("({})", expression_snapshot(inner)),
            other => panic!("snapshot helper does not support {other:?}"),
        }
    }

    fn function_snapshot(function: &crate::ast::FunctionDeclaration) -> String {
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| {
                format!(
                    "{}: {}",
                    parameter.name.name,
                    type_reference_snapshot(&parameter.type_reference)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let return_type = function
            .return_type
            .as_ref()
            .map(|type_reference| format!(" -> {}", type_reference_snapshot(type_reference)))
            .unwrap_or_default();
        let StatementKind::Return(Some(value)) = &only_statement(function).kind else {
            panic!("snapshot function should contain a value return");
        };
        format!(
            "fn {}({parameters}){return_type}\n  return {}\n",
            function.name.name,
            expression_snapshot(value)
        )
    }

    #[test]
    fn function_ast_matches_the_golden_snapshot() {
        let result = parse_text("add.prnc", "fn add(a: Int, b: Int) -> Int { return a + b }");
        let function = only_function(&result);
        assert_eq!(
            function_snapshot(function),
            include_str!("../tests/snapshots/add.ast")
        );
    }

    #[test]
    fn source_extensions_produce_identical_ast_and_preserve_spans() {
        let text = "fn main() { let value: Int = 42 }";
        let prnc = parse_text("hello.prnc", text);
        let princi = parse_text("hello.princi", text);

        assert!(prnc.diagnostics.is_empty());
        assert!(princi.diagnostics.is_empty());
        assert_eq!(prnc.program, princi.program);
        assert_eq!(prnc.program.span.start, 0);
        assert_eq!(prnc.program.span.end, text.len());
        let Declaration::Function(function) = &prnc.program.declarations[0] else {
            panic!("expected function declaration");
        };
        let StatementKind::Variable(variable) = &function.body.statements[0].kind else {
            panic!("expected variable declaration");
        };
        assert_eq!(variable.name.span, crate::source::SourceSpan::new(16, 21));
    }

    #[test]
    fn parses_control_flow_assignments_lists_calls_and_ranges() {
        let result = parse_text(
            "main.prnc",
            r#"fn main() {
    var values: List<Int> = [1, 2, 3]
    let first = values[0]
    values[1] += -2
    if ready {
        print("large")
    } else {
        print("small")
    }
    while ready && first < 5 { first = first + 1 }
    for index in 0..10 { print(index) }
    return
}"#,
        );
        let function = only_function(&result);

        assert_eq!(function.body.statements.len(), 7);
        assert!(matches!(
            function.body.statements[0].kind,
            StatementKind::Variable(_)
        ));
        assert!(matches!(
            function.body.statements[1].kind,
            StatementKind::Variable(_)
        ));
        assert!(matches!(
            function.body.statements[2].kind,
            StatementKind::Assignment(_)
        ));
        let StatementKind::If(if_statement) = &function.body.statements[3].kind else {
            panic!("expected if statement");
        };
        assert!(matches!(
            if_statement.condition.kind,
            ExpressionKind::Identifier(_)
        ));
        assert!(if_statement.else_branch.is_some());
        assert!(matches!(
            function.body.statements[4].kind,
            StatementKind::While(_)
        ));
        let StatementKind::For(for_statement) = &function.body.statements[5].kind else {
            panic!("expected for statement");
        };
        assert!(matches!(
            for_statement.range.kind,
            ExpressionKind::Range { .. }
        ));
        assert!(matches!(
            function.body.statements[6].kind,
            StatementKind::Return(None)
        ));
    }

    #[test]
    fn parses_class_and_struct_members_and_object_construction() {
        let result = parse_text(
            "types.princi",
            r#"class User {
    name: String
    age: Int
    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }
    fn greet() { print("Hello") }
}
struct Point {
    x: Float
    y: Float
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.program.declarations.len(), 2);
        let Declaration::Class(user) = &result.program.declarations[0] else {
            panic!("expected class declaration");
        };
        assert_eq!(user.kind, TypeDeclarationKind::Class);
        assert_eq!(user.members.len(), 4);
        assert!(matches!(user.members[0], ClassMember::Field(_)));
        assert!(matches!(user.members[2], ClassMember::Initializer(_)));
        assert!(matches!(user.members[3], ClassMember::Method(_)));
        let Declaration::Struct(point) = &result.program.declarations[1] else {
            panic!("expected struct declaration");
        };
        assert_eq!(point.kind, TypeDeclarationKind::Struct);
        assert_eq!(point.members.len(), 2);

        let construction = parse_text(
            "construct.prnc",
            "fn main() { let user = User { name: \"Ada\", age: 36 } }",
        );
        let function = only_function(&construction);
        let StatementKind::Variable(variable) = &only_statement(function).kind else {
            panic!("expected variable");
        };
        assert!(matches!(
            variable.initializer.as_ref().map(|value| &value.kind),
            Some(ExpressionKind::Construction { .. })
        ));
    }

    #[test]
    fn binary_operators_follow_declared_precedence_and_associate_left() {
        let result = parse_text(
            "precedence.prnc",
            "fn main() { let value = 1 + 2 * 3 == 7 || false && true }",
        );
        let function = only_function(&result);
        let StatementKind::Variable(variable) = &only_statement(function).kind else {
            panic!("expected variable");
        };
        let Some(Expression {
            kind:
                ExpressionKind::Binary {
                    operator: BinaryOperator::Or,
                    left,
                    right,
                },
            ..
        }) = variable.initializer.as_ref()
        else {
            panic!("logical-or should be the root expression");
        };
        assert!(matches!(
            left.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Equal,
                ..
            }
        ));
        assert!(matches!(
            right.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::And,
                ..
            }
        ));

        let left_associative = parse_text("assoc.prnc", "fn main() { let n = 8 - 3 - 1 }");
        let function = only_function(&left_associative);
        let StatementKind::Variable(variable) = &only_statement(function).kind else {
            panic!("expected variable");
        };
        let ExpressionKind::Binary { left, .. } = &variable.initializer.as_ref().unwrap().kind
        else {
            panic!("expected binary expression");
        };
        assert!(matches!(
            left.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Subtract,
                ..
            }
        ));
    }

    #[test]
    fn parses_nested_list_type_arguments_and_preserves_the_full_span() {
        let result = parse_text(
            "lists.prnc",
            "fn first(values: List<List<Int>>) -> List<Int> { return values[0] }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Declaration::Function(function) = &result.program.declarations[0] else {
            panic!("expected function declaration");
        };
        assert_eq!(
            type_reference_snapshot(&function.parameters[0].type_reference),
            "List<List<Int>>"
        );
        assert_eq!(
            type_reference_snapshot(function.return_type.as_ref().unwrap()),
            "List<Int>"
        );
        assert_eq!(
            &"fn first(values: List<List<Int>>) -> List<Int> { return values[0] }"[function
                .parameters[0]
                .type_reference
                .span
                .start
                ..function.parameters[0].type_reference.span.end],
            "List<List<Int>>"
        );
    }

    #[test]
    fn parser_reports_multiple_positioned_errors_and_recovers() {
        let result = parse_text(
            "broken.prnc",
            "fn main() {\n    let = 1\n    let second = ;\n    print(\"continued\")\n}",
        );

        assert_eq!(result.diagnostics.len(), 2, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].location().unwrap().line, 2);
        assert_eq!(result.diagnostics[1].location().unwrap().line, 3);
        assert!(result.diagnostics[0]
            .to_string()
            .contains("broken.prnc:2:9: error: expected variable name"));
        let Declaration::Function(function) = &result.program.declarations[0] else {
            panic!("parser should preserve the enclosing function");
        };
        assert!(function.body.statements.iter().any(|statement| matches!(
            &statement.kind,
            StatementKind::Expression(Expression {
                kind: ExpressionKind::Call { .. },
                ..
            })
        )));
    }

    #[test]
    fn semicolons_and_parenthesized_ranges_are_supported() {
        let result = parse_text(
            "separators.prnc",
            "fn main() { let x = 1; for i in (0..3) { print(i); }; }",
        );
        let function = only_function(&result);
        assert_eq!(function.body.statements.len(), 2);
        assert!(matches!(
            function.body.statements[1].kind,
            StatementKind::For(_)
        ));
    }
}
