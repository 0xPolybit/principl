use crate::ast::*;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
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
            TokenKind::Identifier(ref name) if name == "enum" => {
                self.parse_enum().map(Declaration::Enum)
            }
            TokenKind::Keyword(Keyword::Interface) => {
                self.parse_interface().map(Declaration::Interface)
            }
            TokenKind::Keyword(Keyword::Trait) => {
                self.parse_trait().map(Declaration::Trait)
            }
            TokenKind::Keyword(Keyword::Import) => self.parse_import().map(Declaration::Import),
            TokenKind::Keyword(Keyword::Extern) => {
                self.parse_extern_block().map(Declaration::ExternBlock)
            }
            _ => Err(self.error_here(
                "expected a function, class, struct, enum, interface, trait, import, or extern declaration",
            )),
        }
    }

    fn parse_enum(&mut self) -> Result<EnumDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected enum name")?;
        let type_parameters = self.parse_type_parameters()?;
        self.expect_punctuation(Punctuation::LeftBrace, "expected '{' after enum name")?;

        let mut variants = Vec::new();
        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let variant_name = self.expect_identifier("expected enum variant name")?;
            let variant_start = variant_name.span.start;
            let mut payload = Vec::new();
            let mut end = variant_name.span.end;
            if self.eat_punctuation(Punctuation::LeftParen) {
                if !self.check_punctuation(Punctuation::RightParen) {
                    loop {
                        payload.push(self.parse_type_reference("expected enum payload type")?);
                        if !self.eat_punctuation(Punctuation::Comma) {
                            break;
                        }
                        if self.check_punctuation(Punctuation::RightParen) {
                            break;
                        }
                    }
                }
                let close = self.expect_punctuation(
                    Punctuation::RightParen,
                    "expected ')' after enum payload types",
                )?;
                end = close.location.span.end;
            }
            variants.push(EnumVariantDeclaration {
                name: variant_name,
                payload,
                span: SourceSpan::new(variant_start, end),
            });

            // Newlines are insignificant; commas and semicolons are accepted
            // as optional separators for single-line declarations.
            self.eat_punctuation(Punctuation::Comma);
            self.eat_punctuation(Punctuation::Semicolon);
        }

        let close = self.expect_punctuation(
            Punctuation::RightBrace,
            "expected '}' to close enum declaration",
        )?;
        Ok(EnumDeclaration {
            name,
            type_parameters,
            variants,
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_extern_block(&mut self) -> Result<ExternBlockDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let abi_token = self.current().clone();
        let abi = match abi_token.kind {
            TokenKind::StringLiteral(abi) => {
                self.advance();
                abi
            }
            _ => return Err(self.error_here("expected ABI string after 'extern'")),
        };
        self.expect_punctuation(
            Punctuation::LeftBrace,
            "expected '{' after extern ABI string",
        )?;

        let mut functions = Vec::new();
        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let before = self.cursor;
            if self.check_keyword(Keyword::Fn) {
                match self.parse_external_function() {
                    Ok(function) => functions.push(function),
                    Err(diagnostic) => {
                        self.diagnostics.push(diagnostic);
                        self.recover_extern_member(before);
                    }
                }
            } else {
                self.diagnostics
                    .push(self.error_here("extern blocks may contain only function declarations"));
                self.recover_extern_member(before);
            }
        }

        let close = self.expect_punctuation(
            Punctuation::RightBrace,
            "expected '}' after extern declarations",
        )?;
        Ok(ExternBlockDeclaration {
            abi,
            abi_span: abi_token.location.span,
            functions,
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_external_function(&mut self) -> Result<ExternalFunctionDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected external function name")?;
        let parameters = self.parse_parameters()?;
        let return_type = if self.eat_punctuation(Punctuation::Arrow) {
            Some(self.parse_type_reference("expected external return type after '->'")?)
        } else {
            None
        };
        self.finish_statement()?;
        let end = return_type.as_ref().map_or_else(
            || self.tokens[self.cursor.saturating_sub(1)].location.span.end,
            |return_type| return_type.span.end,
        );
        Ok(ExternalFunctionDeclaration {
            name,
            parameters,
            return_type,
            span: SourceSpan::new(start, end),
        })
    }

    fn recover_extern_member(&mut self, before: usize) {
        let mut advanced = self.cursor > before;
        if !advanced && !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            self.advance();
            advanced = true;
        }

        while !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            if self.eat_punctuation(Punctuation::Semicolon) {
                return;
            }
            if advanced && self.check_keyword(Keyword::Fn) {
                return;
            }
            self.advance();
            advanced = true;
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected function name")?;
        let type_parameters = self.parse_type_parameters()?;
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
            type_parameters,
            parameters,
            return_type,
            body,
            span,
        })
    }

    fn parse_type_parameters(&mut self) -> Result<Vec<TypeParameterDeclaration>, Diagnostic> {
        if !self.eat_operator(Operator::Less) {
            return Ok(Vec::new());
        }

        let mut parameters = Vec::new();
        loop {
            let name = self.expect_identifier("expected generic parameter name")?;
            let mut bounds = Vec::new();
            if self.eat_punctuation(Punctuation::Colon) {
                loop {
                    bounds.push(self.parse_type_reference("expected interface or trait bound")?);
                    if !self.eat_operator(Operator::Plus) {
                        break;
                    }
                }
            }
            parameters.push(TypeParameterDeclaration { name, bounds });
            if self.eat_punctuation(Punctuation::Comma) {
                continue;
            }
            let close = self.current().clone();
            if close.kind != TokenKind::Operator(Operator::Greater) {
                return Err(Diagnostic::at_source(
                    DiagnosticCode::Syntax,
                    "expected '>' after generic parameters",
                    self.source,
                    close.location.span,
                ));
            }
            self.advance();
            break;
        }
        Ok(parameters)
    }

    fn parse_type_arguments(&mut self) -> Result<(Vec<TypeReference>, SourceSpan), Diagnostic> {
        let open = self.expect_operator(Operator::Less, "expected '<' before type arguments")?;
        let mut arguments = Vec::new();
        loop {
            arguments.push(self.parse_type_reference("expected generic type argument")?);
            if self.eat_punctuation(Punctuation::Comma) {
                continue;
            }
            let close = self.current().clone();
            if close.kind != TokenKind::Operator(Operator::Greater) {
                return Err(Diagnostic::at_source(
                    DiagnosticCode::Syntax,
                    "expected '>' after type arguments",
                    self.source,
                    close.location.span,
                ));
            }
            self.advance();
            return Ok((
                arguments,
                SourceSpan::new(open.location.span.start, close.location.span.end),
            ));
        }
    }

    fn expect_operator(&mut self, operator: Operator, message: &str) -> Result<Token, Diagnostic> {
        if self.eat_operator(operator) {
            Ok(self.tokens[self.cursor - 1].clone())
        } else {
            Err(self.error_here(message))
        }
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
        let mut end = name.span.end;
        let (arguments, span) = if self.check_operator(Operator::Less) {
            let (arguments, span) = self.parse_type_arguments()?;
            end = span.end;
            (arguments, span)
        } else {
            (Vec::new(), SourceSpan::new(name.span.start, end))
        };
        Ok(TypeReference {
            name: name.name,
            arguments,
            span: SourceSpan::new(name.span.start, span.end.max(end)),
        })
    }

    fn parse_type_declaration(
        &mut self,
        kind: TypeDeclarationKind,
    ) -> Result<TypeDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected class or struct name")?;
        let type_parameters = self.parse_type_parameters()?;
        let mut interfaces = Vec::new();
        if self.eat_keyword(Keyword::Implements) {
            loop {
                interfaces
                    .push(self.parse_type_reference("expected interface name after 'implements'")?);
                if !self.eat_punctuation(Punctuation::Comma) {
                    break;
                }
            }
        }
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
            type_parameters,
            interfaces,
            members,
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_interface(&mut self) -> Result<InterfaceDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected interface name")?;
        self.expect_punctuation(Punctuation::LeftBrace, "expected '{' after interface name")?;
        let mut methods = Vec::new();
        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let before = self.cursor;
            match self.parse_interface_method() {
                Ok(method) => methods.push(method),
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.recover_interface_member(before);
                }
            }
            if self.cursor == before && !self.at_eof() {
                self.advance();
            }
        }
        let close =
            self.expect_punctuation(Punctuation::RightBrace, "expected '}' to close interface")?;
        Ok(InterfaceDeclaration {
            name,
            methods,
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_interface_method(&mut self) -> Result<InterfaceMethodDeclaration, Diagnostic> {
        let start = self
            .expect_keyword(Keyword::Fn, "interfaces may contain only method signatures")?
            .location
            .span
            .start;
        let name = self.expect_identifier("expected interface method name")?;
        if self.check_operator(Operator::Less) {
            return Err(
                self.error_here("interface methods cannot declare generic parameters in v0.2")
            );
        }
        let parameters = self.parse_parameters()?;
        let return_type = if self.eat_punctuation(Punctuation::Arrow) {
            Some(self.parse_type_reference("expected return type after '->'")?)
        } else {
            None
        };
        if self.check_punctuation(Punctuation::LeftBrace) {
            return Err(self.error_here("interface method declarations cannot have bodies in v0.2"));
        }
        self.finish_member()?;
        let end = return_type.as_ref().map_or_else(
            || self.tokens[self.cursor.saturating_sub(1)].location.span.end,
            |return_type| return_type.span.end,
        );
        Ok(InterfaceMethodDeclaration {
            name,
            parameters,
            return_type,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_trait(&mut self) -> Result<TraitDeclaration, Diagnostic> {
        let start = self.advance().location.span.start;
        let name = self.expect_identifier("expected trait name")?;
        self.expect_punctuation(Punctuation::LeftBrace, "expected '{' after trait name")?;
        let mut methods = Vec::new();
        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let before = self.cursor;
            match self.parse_trait_method() {
                Ok(method) => methods.push(method),
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.recover_interface_member(before);
                }
            }
            if self.cursor == before && !self.at_eof() {
                self.advance();
            }
        }
        let close =
            self.expect_punctuation(Punctuation::RightBrace, "expected '}' to close trait")?;
        Ok(TraitDeclaration {
            name,
            methods,
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_trait_method(&mut self) -> Result<TraitMethodDeclaration, Diagnostic> {
        let start = self
            .expect_keyword(Keyword::Fn, "traits may contain only method declarations")?
            .location
            .span
            .start;
        let name = self.expect_identifier("expected trait method name")?;
        if self.check_operator(Operator::Less) {
            return Err(self.error_here("trait methods cannot declare generic parameters in v0.2"));
        }
        let parameters = self.parse_parameters()?;
        let return_type = if self.eat_punctuation(Punctuation::Arrow) {
            Some(self.parse_type_reference("expected return type after '->'")?)
        } else {
            None
        };
        let body = if self.check_punctuation(Punctuation::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            self.finish_member()?;
            None
        };
        let end = body.as_ref().map_or_else(
            || {
                return_type.as_ref().map_or_else(
                    || self.tokens[self.cursor.saturating_sub(1)].location.span.end,
                    |ty| ty.span.end,
                )
            },
            |body| body.span.end,
        );
        Ok(TraitMethodDeclaration {
            name,
            parameters,
            return_type,
            body,
            span: SourceSpan::new(start, end),
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
            return Err(Diagnostic::at_source(
                DiagnosticCode::Syntax,
                "variable declaration needs a type or an initializer",
                self.source,
                name.span,
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
                return Err(Diagnostic::at_source(
                    DiagnosticCode::Syntax,
                    "invalid assignment target",
                    self.source,
                    target_or_expression.span,
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
            return Err(Diagnostic::at_source(
                DiagnosticCode::Syntax,
                "expected a range expression after 'in'",
                self.source,
                range.span,
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
            if self.check_operator(Operator::Question) {
                let question = self.advance();
                left = Expression {
                    span: SourceSpan::new(left.span.start, question.location.span.end),
                    kind: ExpressionKind::Try(Box::new(left)),
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
                if name == "match" {
                    let saved = self.cursor;
                    let diagnostic_count = self.diagnostics.len();
                    let start = token.location.span.start;
                    self.advance();
                    if let Ok(value) = self.parse_expression_bp(RANGE_PRECEDENCE, false) {
                        if self.check_punctuation(Punctuation::LeftBrace) {
                            return self.parse_match_expression_after_value(start, value);
                        }
                    }
                    self.cursor = saved;
                    self.diagnostics.truncate(diagnostic_count);
                }
                self.advance();
                let identifier = Identifier {
                    name,
                    span: token.location.span,
                };
                if self.check_operator(Operator::Less) {
                    let after_name = self.cursor;
                    if let Ok((arguments, argument_span)) = self.parse_type_arguments() {
                        if self.check_punctuation(Punctuation::LeftBrace) && allow_construction {
                            let type_reference = TypeReference {
                                name: identifier.name.clone(),
                                arguments,
                                span: SourceSpan::new(identifier.span.start, argument_span.end),
                            };
                            return self.parse_construction_expression(type_reference);
                        }
                        if self.check_punctuation(Punctuation::LeftParen)
                            || self.check_punctuation(Punctuation::Dot)
                        {
                            let span = SourceSpan::new(identifier.span.start, argument_span.end);
                            return Ok(Expression {
                                kind: ExpressionKind::GenericReference {
                                    name: identifier,
                                    arguments,
                                },
                                span,
                            });
                        }
                    }
                    self.cursor = after_name;
                }
                if allow_construction && self.check_punctuation(Punctuation::LeftBrace) {
                    self.parse_construction_expression(TypeReference {
                        name: identifier.name,
                        arguments: Vec::new(),
                        span: identifier.span,
                    })
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

    fn parse_match_expression_after_value(
        &mut self,
        start: usize,
        value: Expression,
    ) -> Result<Expression, Diagnostic> {
        self.expect_punctuation(Punctuation::LeftBrace, "expected '{' before match arms")?;
        let mut arms = Vec::new();

        while !self.check_punctuation(Punctuation::RightBrace) && !self.at_eof() {
            let arm_start = self.current().location.span.start;
            let pattern = self.parse_pattern()?;
            self.expect_punctuation(Punctuation::FatArrow, "expected '=>' after match pattern")?;
            let body = if self.check_punctuation(Punctuation::LeftBrace) {
                MatchArmBody::Block(self.parse_block()?)
            } else {
                MatchArmBody::Expression(self.parse_expression()?)
            };
            let arm_end = match &body {
                MatchArmBody::Expression(expression) => expression.span.end,
                MatchArmBody::Block(block) => block.span.end,
            };
            arms.push(MatchArm {
                pattern,
                body,
                span: SourceSpan::new(arm_start, arm_end),
            });

            if self.eat_punctuation(Punctuation::Comma)
                || self.eat_punctuation(Punctuation::Semicolon)
                || self.check_punctuation(Punctuation::RightBrace)
                || self.line_break_before_current()
                || self.can_start_pattern()
            {
                continue;
            }
            return Err(self.error_here("expected a line break or separator after match arm"));
        }

        let close =
            self.expect_punctuation(Punctuation::RightBrace, "expected '}' after match arms")?;
        Ok(Expression {
            kind: ExpressionKind::Match(MatchExpression {
                value: Box::new(value),
                arms,
            }),
            span: SourceSpan::new(start, close.location.span.end),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, Diagnostic> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::Identifier(name) => {
                self.advance();
                if name == "_" {
                    return Ok(Pattern {
                        kind: PatternKind::Wildcard,
                        span: token.location.span,
                    });
                }
                let identifier = Identifier {
                    name,
                    span: token.location.span,
                };
                if self.eat_punctuation(Punctuation::LeftParen) {
                    let mut payload = Vec::new();
                    if !self.check_punctuation(Punctuation::RightParen) {
                        loop {
                            payload.push(self.parse_pattern()?);
                            if !self.eat_punctuation(Punctuation::Comma) {
                                break;
                            }
                            if self.check_punctuation(Punctuation::RightParen) {
                                break;
                            }
                        }
                    }
                    let close = self.expect_punctuation(
                        Punctuation::RightParen,
                        "expected ')' after variant pattern",
                    )?;
                    Ok(Pattern {
                        kind: PatternKind::Variant {
                            name: identifier,
                            payload,
                        },
                        span: SourceSpan::new(token.location.span.start, close.location.span.end),
                    })
                } else {
                    Ok(Pattern {
                        kind: PatternKind::Name(identifier),
                        span: token.location.span,
                    })
                }
            }
            TokenKind::IntegerLiteral(value) => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Integer(value)),
                    span: token.location.span,
                })
            }
            TokenKind::FloatingPointLiteral(value) => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::FloatingPoint(value)),
                    span: token.location.span,
                })
            }
            TokenKind::StringLiteral(value) => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::String(value)),
                    span: token.location.span,
                })
            }
            TokenKind::BooleanLiteral(value) => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Boolean(value)),
                    span: token.location.span,
                })
            }
            TokenKind::Operator(Operator::Minus) => {
                self.advance();
                let number = self.current().clone();
                match number.kind {
                    TokenKind::IntegerLiteral(value) => {
                        self.advance();
                        Ok(Pattern {
                            kind: PatternKind::Literal(Literal::Integer(format!("-{value}"))),
                            span: SourceSpan::new(
                                token.location.span.start,
                                number.location.span.end,
                            ),
                        })
                    }
                    TokenKind::FloatingPointLiteral(value) => {
                        self.advance();
                        Ok(Pattern {
                            kind: PatternKind::Literal(Literal::FloatingPoint(format!("-{value}"))),
                            span: SourceSpan::new(
                                token.location.span.start,
                                number.location.span.end,
                            ),
                        })
                    }
                    _ => Err(self.error_here("expected a numeric literal after '-' in pattern")),
                }
            }
            _ => Err(self.error_here("expected a match pattern")),
        }
    }

    fn can_start_pattern(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Identifier(_)
                | TokenKind::IntegerLiteral(_)
                | TokenKind::FloatingPointLiteral(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::BooleanLiteral(_)
                | TokenKind::Operator(Operator::Minus)
        )
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
        type_reference: TypeReference,
    ) -> Result<Expression, Diagnostic> {
        self.advance();
        let start = type_reference.span.start;
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
            kind: ExpressionKind::Construction {
                type_reference,
                fields,
            },
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

    fn recover_interface_member(&mut self, before: usize) {
        let mut advanced = self.cursor > before;
        let mut body_depth = 0usize;
        if !advanced && !self.at_eof() && !self.check_punctuation(Punctuation::RightBrace) {
            self.advance();
            advanced = true;
        }

        while !self.at_eof() {
            if body_depth == 0 && self.check_punctuation(Punctuation::RightBrace) {
                return;
            }
            if body_depth == 0
                && advanced
                && self.line_break_before_current()
                && self.can_start_class_member()
            {
                return;
            }
            if body_depth == 0
                && (self.eat_punctuation(Punctuation::Semicolon)
                    || self.eat_punctuation(Punctuation::Comma))
            {
                return;
            }
            match self.current().kind {
                TokenKind::Punctuation(Punctuation::LeftBrace) => body_depth += 1,
                TokenKind::Punctuation(Punctuation::RightBrace) if body_depth > 0 => {
                    body_depth -= 1;
                }
                _ => {}
            }
            self.advance();
            advanced = true;
        }
    }

    fn is_declaration_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Keyword(
                Keyword::Fn
                    | Keyword::Class
                    | Keyword::Struct
                    | Keyword::Interface
                    | Keyword::Trait
                    | Keyword::Import
                    | Keyword::Extern
            )
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

    fn check_operator(&self, operator: Operator) -> bool {
        self.current().kind == TokenKind::Operator(operator)
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
        Diagnostic::at_source(
            DiagnosticCode::Syntax,
            message,
            self.source,
            self.current().location.span,
        )
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
        StatementKind, TypeDeclarationKind, VariableDeclaration,
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
            ExpressionKind::Try(inner) => format!("{}?", expression_snapshot(inner)),
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
        let expected = include_str!("../tests/snapshots/add.ast").replace("\r\n", "\n");
        assert_eq!(function_snapshot(function), expected);
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
    fn parses_c_ffi_declarations_without_function_bodies() {
        let result = parse_text(
            "ffi.prnc",
            "extern \"C\" {\n    fn abs(value: Int32) -> Int32\n    fn get_count() -> Int64;\n    fn reset()\n}\nfn main() {}",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let [Declaration::ExternBlock(block), Declaration::Function(main)] =
            result.program.declarations.as_slice()
        else {
            panic!("expected the extern block followed by main");
        };
        assert_eq!(block.abi, "C");
        assert_eq!(block.functions.len(), 3);
        assert_eq!(block.functions[0].name.name, "abs");
        assert_eq!(
            block.functions[0].parameters[0].type_reference.name,
            "Int32"
        );
        assert_eq!(
            block.functions[0]
                .return_type
                .as_ref()
                .expect("return type")
                .name,
            "Int32"
        );
        assert_eq!(block.functions[2].name.name, "reset");
        assert!(block.functions[2].return_type.is_none());
        assert_eq!(main.name.name, "main");
    }

    #[test]
    fn recovers_from_malformed_c_ffi_members() {
        let result = parse_text(
            "ffi-recovery.prnc",
            "extern \"C\" {\n    fn broken(value: )\n    fn good() -> Int32\n}\nfn main() {}",
        );
        assert!(!result.diagnostics.is_empty());
        assert!(result.diagnostics[0]
            .to_string()
            .starts_with("ffi-recovery.prnc:2:"));
        assert!(result.program.declarations.iter().any(
            |declaration| matches!(declaration, Declaration::Function(function) if function.name.name == "main")
        ));
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
    fn parses_match_statement_arms_payload_patterns_and_expression_arms() {
        let result = parse_text(
            "match.prnc",
            r#"enum State { Connected Connecting Failed(String) }
fn main() {
    let state = State.Connected
    match state {
        Connected => println("online")
        Failed(reason) => println(reason)
        _ => println("offline")
    }
    let message = match state {
        Connected => "online"
        Connecting => "connecting"
        Failed(reason) => reason
    }
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Declaration::Function(function) = &result.program.declarations[1] else {
            panic!("expected main function");
        };
        let StatementKind::Expression(Expression {
            kind: ExpressionKind::Match(statement_match),
            ..
        }) = &function.body.statements[1].kind
        else {
            panic!("match statement should use the match expression AST");
        };
        assert_eq!(statement_match.arms.len(), 3);
        assert!(matches!(
            statement_match.arms[1].pattern.kind,
            crate::ast::PatternKind::Variant { .. }
        ));
        let StatementKind::Variable(variable) = &function.body.statements[2].kind else {
            panic!("expected match result binding");
        };
        assert!(matches!(
            variable.initializer.as_ref().unwrap().kind,
            ExpressionKind::Match(_)
        ));
    }

    #[test]
    fn match_remains_available_as_a_contextual_identifier() {
        let result = parse_text(
            "match-name.prnc",
            "fn match(value: Int) -> Int { return value }\nfn main() { let result = match(7); let match_value = 2 }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Declaration::Function(function) = &result.program.declarations[0] else {
            panic!("expected function named match");
        };
        assert_eq!(function.name.name, "match");
        let Declaration::Function(main) = &result.program.declarations[1] else {
            panic!("expected main function");
        };
        assert_eq!(main.body.statements.len(), 2);
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
    fn parses_interfaces_and_multiple_class_implementations() {
        let result = parse_text(
            "interfaces.prnc",
            r#"interface Drawable {
    fn draw(style: String) -> String
    fn visible() -> Bool;
}
interface Clickable { fn click() }
class Widget implements Drawable, Clickable {
    fn draw(style: String) -> String { return style }
    fn visible() -> Bool { return true }
    fn click() {}
}
fn main() {}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Declaration::Interface(drawable) = &result.program.declarations[0] else {
            panic!("expected interface declaration");
        };
        assert_eq!(drawable.methods.len(), 2);
        assert_eq!(drawable.methods[0].name.name, "draw");
        assert_eq!(
            drawable.methods[0].parameters[0].type_reference.name,
            "String"
        );
        assert_eq!(
            drawable.methods[0].return_type.as_ref().unwrap().name,
            "String"
        );
        let Declaration::Class(widget) = &result.program.declarations[2] else {
            panic!("expected class declaration");
        };
        assert_eq!(widget.interfaces.len(), 2);
        assert_eq!(widget.interfaces[0].name, "Drawable");
        assert_eq!(widget.interfaces[1].name, "Clickable");
    }

    #[test]
    fn parses_traits_with_required_and_default_methods() {
        let result = parse_text(
            "traits.prnc",
            r#"trait Named {
    fn name() -> String

    fn describe() {
        println(self.name())
    }
}
struct Label implements Named {
    value: String
    fn name() -> String { return self.value }
}
fn main() {}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Declaration::Trait(named) = &result.program.declarations[0] else {
            panic!("expected trait declaration");
        };
        assert_eq!(named.methods.len(), 2);
        assert!(named.methods[0].body.is_none());
        assert!(named.methods[1].body.is_some());
        let Declaration::Struct(label) = &result.program.declarations[1] else {
            panic!("expected struct declaration");
        };
        assert_eq!(label.interfaces[0].name, "Named");
    }

    #[test]
    fn trait_declarations_reject_fields_constructors_and_generic_methods() {
        let result = parse_text(
            "invalid-trait.prnc",
            "trait Bad { value: Int; init() {}; fn generic<T>(value: T) }\nfn main() {}",
        );
        assert!(!result.diagnostics.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message().contains("only method declarations")));
        let generic = parse_text(
            "generic-trait-method.prnc",
            "trait Bad { fn generic<T>(value: T) }\nfn main() {}",
        );
        assert!(generic
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message().contains("generic parameters")));
    }

    #[test]
    fn rejects_interface_fields_constructors_bodies_and_generic_methods() {
        let result = parse_text(
            "invalid-interface.prnc",
            r#"interface Bad {
    value: Int
    init() {}
    fn body() {}
}
fn main() {}"#,
        );
        assert!(!result.diagnostics.is_empty());
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic
                .location()
                .is_some_and(|location| location.line > 1)
        }));
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message().contains("cannot have bodies")));
        let generic = parse_text(
            "generic-interface-method.prnc",
            "interface Bad { fn generic<T>(value: T) }\nfn main() {}",
        );
        assert!(generic
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message().contains("generic parameters")));
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
    fn postfix_question_binds_tighter_than_binary_operators_and_keeps_its_span() {
        let result = parse_text("try.prnc", "fn main() { let value = read()? + 1 }");
        let function = only_function(&result);
        let StatementKind::Variable(variable) = &function.body.statements[0].kind else {
            panic!("expected a variable declaration");
        };
        let ExpressionKind::Binary { left, .. } = &variable.initializer.as_ref().unwrap().kind
        else {
            panic!("the binary expression should be the initializer");
        };
        let ExpressionKind::Try(operand) = &left.kind else {
            panic!("postfix '?' should be attached before addition");
        };
        assert!(matches!(operand.kind, ExpressionKind::Call { .. }));
        assert_eq!(left.span.end, left.span.start + "read()?".len());
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
    fn parses_simple_payload_and_generic_enum_declarations_and_construction_syntax() {
        let result = parse_text(
            "enums.prnc",
            r#"enum Direction { North South East West }
enum NetworkState {
    Connected
    Failed(String, Int)
}
enum Maybe<T> { Some(T) None }
enum Result<T, E> { Ok(T) Err(E) }
fn main() {
    let direction = Direction.North
    let failure = NetworkState.Failed("offline", 7)
    let result = Result<Int, String>.Ok(42)
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

        let Declaration::Enum(direction) = &result.program.declarations[0] else {
            panic!("expected enum declaration");
        };
        assert_eq!(
            direction
                .variants
                .iter()
                .map(|variant| variant.name.name.as_str())
                .collect::<Vec<_>>(),
            ["North", "South", "East", "West"]
        );
        let Declaration::Enum(state) = &result.program.declarations[1] else {
            panic!("expected payload enum declaration");
        };
        assert_eq!(state.variants[1].payload.len(), 2);
        assert_eq!(state.variants[1].payload[0].name, "String");
        assert_eq!(state.variants[1].payload[1].name, "Int");
        let Declaration::Enum(maybe) = &result.program.declarations[2] else {
            panic!("expected generic enum declaration");
        };
        assert_eq!(maybe.type_parameters[0].name.name, "T");

        let Declaration::Function(main) = &result.program.declarations[4] else {
            panic!("expected main function");
        };
        let StatementKind::Variable(variable) = &main.body.statements[2].kind else {
            panic!("expected generic enum variable");
        };
        let Some(Expression {
            kind: ExpressionKind::Call { callee, arguments },
            ..
        }) = variable.initializer.as_ref()
        else {
            panic!("expected payload variant call");
        };
        assert_eq!(arguments.len(), 1);
        let ExpressionKind::Member { object, member } = &callee.kind else {
            panic!("expected qualified enum variant");
        };
        assert_eq!(member.name, "Ok");
        assert!(matches!(
            object.kind,
            ExpressionKind::GenericReference { .. }
        ));
    }

    #[test]
    fn enum_programs_parse_identically_with_both_source_extensions() {
        let text = "enum Maybe<T> { Some(T) None }\nfn main() { let enum = 1; let value = Maybe<Int>.Some(enum) }";
        let prnc = parse_text("alias.prnc", text);
        let princi = parse_text("alias.princi", text);
        assert!(prnc.diagnostics.is_empty(), "{:?}", prnc.diagnostics);
        assert!(princi.diagnostics.is_empty(), "{:?}", princi.diagnostics);
        assert_eq!(prnc.program, princi.program);
    }

    #[test]
    fn parses_generic_declarations_nested_types_and_construction_arguments() {
        let text = r#"class Box<T> { value: T }
struct Pair<A, B> {
    first: A
    second: B
}
fn identity<T>(value: T) -> T { return value }
fn take(value: Box<List<Int>>) {}
fn main() {
    let box = Box<Int> { value: 3 }
    let explicit = identity<Int>(box.value)
    let positional = Box<List<Int>>([1, 2])
}"#;
        let result = parse_text("generic.prnc", text);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

        let Declaration::Class(box_declaration) = &result.program.declarations[0] else {
            panic!("expected generic class declaration");
        };
        assert_eq!(box_declaration.type_parameters.len(), 1);
        assert_eq!(box_declaration.type_parameters[0].name.name, "T");
        let Declaration::Struct(pair_declaration) = &result.program.declarations[1] else {
            panic!("expected generic struct declaration");
        };
        assert_eq!(
            pair_declaration
                .type_parameters
                .iter()
                .map(|parameter| parameter.name.name.as_str())
                .collect::<Vec<_>>(),
            ["A", "B"]
        );
        let Declaration::Function(identity) = &result.program.declarations[2] else {
            panic!("expected generic function declaration");
        };
        assert_eq!(identity.type_parameters[0].name.name, "T");
        assert_eq!(identity.parameters[0].type_reference.name, "T");
        assert_eq!(identity.return_type.as_ref().unwrap().name, "T");
        let Declaration::Function(take) = &result.program.declarations[3] else {
            panic!("expected nested generic function declaration");
        };
        assert_eq!(
            type_reference_snapshot(&take.parameters[0].type_reference),
            "Box<List<Int>>"
        );
        let Declaration::Function(main) = &result.program.declarations[4] else {
            panic!("expected main function");
        };
        assert!(matches!(
            &main.body.statements[0],
            Statement { kind: StatementKind::Variable(VariableDeclaration { initializer: Some(Expression { kind: ExpressionKind::Construction { type_reference, .. }, .. }), .. }), .. }
                if type_reference.name == "Box" && type_reference.arguments.len() == 1
        ));
        assert!(matches!(
            &main.body.statements[1],
            Statement { kind: StatementKind::Variable(VariableDeclaration { initializer: Some(Expression { kind: ExpressionKind::Call { callee, .. }, .. }), .. }), .. }
                if matches!(&callee.kind, ExpressionKind::GenericReference { name, arguments } if name.name == "identity" && arguments.len() == 1)
        ));
        assert!(matches!(
            &main.body.statements[2],
            Statement { kind: StatementKind::Variable(VariableDeclaration { initializer: Some(Expression { kind: ExpressionKind::Call { callee, .. }, .. }), .. }), .. }
                if matches!(&callee.kind, ExpressionKind::GenericReference { name, arguments } if name.name == "Box" && type_reference_snapshot(&arguments[0]) == "List<Int>")
        ));
    }

    #[test]
    fn parses_single_and_multiple_generic_contract_bounds() {
        let result = parse_text(
            "bounds.prnc",
            "trait Named { fn name() -> String }\ninterface Drawable { fn draw() }\nfn show<T: Named + Drawable, U: Named>(value: T, other: U) {}\nclass Repository<T: Named + Drawable> {}\nstruct Holder<T: Named> { value: T }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let Declaration::Function(show) = &result.program.declarations[2] else {
            panic!("expected bounded generic function");
        };
        assert_eq!(show.type_parameters[0].name.name, "T");
        assert_eq!(
            show.type_parameters[0]
                .bounds
                .iter()
                .map(|bound| bound.name.as_str())
                .collect::<Vec<_>>(),
            ["Named", "Drawable"]
        );
        assert_eq!(show.type_parameters[1].bounds[0].name, "Named");
        let Declaration::Class(repository) = &result.program.declarations[3] else {
            panic!("expected bounded generic class");
        };
        assert_eq!(repository.type_parameters[0].bounds.len(), 2);
        let Declaration::Struct(holder) = &result.program.declarations[4] else {
            panic!("expected bounded generic struct");
        };
        assert_eq!(holder.type_parameters[0].bounds[0].name, "Named");
    }

    #[test]
    fn generic_syntax_has_identical_ast_for_both_source_extensions() {
        let text = "struct Pair<A, B> {\n first: A\n second: B\n}\nfn identity<T>(value: T) -> T { return value }";
        let prnc = parse_text("pair.prnc", text);
        let princi = parse_text("pair.princi", text);
        assert!(prnc.diagnostics.is_empty(), "{:?}", prnc.diagnostics);
        assert!(princi.diagnostics.is_empty(), "{:?}", princi.diagnostics);
        assert_eq!(prnc.program, princi.program);
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
            .contains("broken.prnc:2:9: error[E0101]: expected variable name"));
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
