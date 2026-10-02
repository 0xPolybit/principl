//! Typed-Princi lowering and the v0.1 Windows native bootstrap backend.
//!
//! The backend emits a self-contained C translation unit from the typed AST,
//! then asks the configured MinGW GCC to compile and link it as a Windows PE
//! executable. Keeping this behind `TypedProgram` makes the native toolchain
//! replaceable without exposing it to the parser or semantic analyzer.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::{
    AssignmentOperator, BinaryOperator, Block, Declaration, Expression, ExpressionKind,
    FunctionDeclaration, Literal, Statement, StatementKind, UnaryOperator,
};
use crate::compiler::{BuildOptions, Target};
use crate::diagnostics::Diagnostic;
use crate::runtime::C_RUNTIME;
use crate::semantic::{GlobalSymbol, TypedProgram};
use crate::source::{SourceFile, SourceSpan};
use crate::types::Type;

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

/// Lower the semantically checked subset to a C translation unit.
pub fn generate_c(typed: &TypedProgram, source: &SourceFile) -> Result<String, Diagnostic> {
    CGenerator::new(typed, source).generate()
}

/// Compile generated C to the requested Windows x86-64 executable.
pub fn compile_native(c_source: &str, options: &BuildOptions) -> Result<(), Diagnostic> {
    if options.compiler.target != Target::WindowsX86_64 {
        return Err(Diagnostic::new(
            "unsupported native target; expected Windows x86-64",
        ));
    }

    let compiler = std::env::var_os("PRINCI_CC").unwrap_or_else(|| OsString::from("gcc"));
    let target = Command::new(&compiler)
        .arg("-dumpmachine")
        .output()
        .map_err(|error| {
            Diagnostic::new(format!(
                "could not start native compiler '{}': {error}; install x86-64 MinGW GCC or set PRINCI_CC",
                compiler.to_string_lossy()
            ))
        })?;
    if !target.status.success() {
        return Err(Diagnostic::new(format!(
            "native compiler '{}' could not report its target: {}",
            compiler.to_string_lossy(),
            process_output(&target)
        )));
    }
    let target_name = String::from_utf8_lossy(&target.stdout);
    if !target_name.trim().starts_with("x86_64-w64-mingw32") {
        return Err(Diagnostic::new(format!(
            "native compiler '{}' targets '{}'; v0.1 requires x86_64-w64-mingw32",
            compiler.to_string_lossy(),
            target_name.trim()
        )));
    }

    let source_file = TemporaryFile::new(".c")?;
    fs::write(&source_file.path, c_source).map_err(|error| {
        Diagnostic::new(format!(
            "could not write temporary native source '{}': {error}",
            source_file.path.display()
        ))
    })?;

    let mut output_file = TemporaryOutput::beside(&options.output_path)?;
    let result = Command::new(&compiler)
        .args(["-std=c11", "-O0", "-fwrapv"])
        .arg(native_tool_path(&source_file.path))
        .arg("-o")
        .arg(native_tool_path(&output_file.path))
        .output()
        .map_err(|error| {
            Diagnostic::new(format!(
                "could not start native compiler '{}': {error}",
                compiler.to_string_lossy()
            ))
        })?;

    if !result.status.success() {
        return Err(Diagnostic::new(format!(
            "native compilation failed with {}:\n{}",
            result.status,
            process_output(&result)
        )));
    }

    if options.output_path.exists() {
        fs::remove_file(&options.output_path).map_err(|error| {
            Diagnostic::new(format!(
                "could not replace output '{}': {error}",
                options.output_path.display()
            ))
        })?;
    }
    fs::rename(&output_file.path, &options.output_path).map_err(|error| {
        Diagnostic::new(format!(
            "could not move executable to '{}': {error}",
            options.output_path.display()
        ))
    })?;
    output_file.keep = true;
    Ok(())
}

fn process_output(output: &std::process::Output) -> String {
    let mut text = String::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.trim().is_empty() {
        text.push_str(stdout.trim());
    }
    if !stderr.trim().is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(stderr.trim());
    }
    if text.is_empty() {
        "the tool produced no diagnostic output".to_owned()
    } else {
        text
    }
}

fn native_tool_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path = path.to_string_lossy();
        if let Some(unc_path) = path.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{unc_path}"));
        }
        if let Some(path) = path.strip_prefix(r"\\?\") {
            return PathBuf::from(path);
        }
        PathBuf::from(path.as_ref())
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn new(extension: &str) -> Result<Self, Diagnostic> {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            path: std::env::temp_dir()
                .join(format!("princi-{}-{id}{extension}", std::process::id())),
        })
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct TemporaryOutput {
    path: PathBuf,
    keep: bool,
}

impl TemporaryOutput {
    fn beside(destination: &Path) -> Result<Self, Diagnostic> {
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let name = destination.file_name().ok_or_else(|| {
            Diagnostic::new(format!("invalid output path '{}'", destination.display()))
        })?;
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = name.to_os_string();
        temporary_name.push(format!(".princi-{}-{id}.tmp.exe", std::process::id()));
        Ok(Self {
            path: parent.join(temporary_name),
            keep: false,
        })
    }
}

impl Drop for TemporaryOutput {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct CGenerator<'a> {
    typed: &'a TypedProgram,
    source: &'a SourceFile,
    output: String,
    scopes: Vec<HashMap<String, String>>,
    next_local: usize,
}

impl<'a> CGenerator<'a> {
    fn new(typed: &'a TypedProgram, source: &'a SourceFile) -> Self {
        Self {
            typed,
            source,
            output: String::new(),
            scopes: Vec::new(),
            next_local: 0,
        }
    }

    fn generate(mut self) -> Result<String, Diagnostic> {
        let functions = self.supported_functions()?;
        let main = functions
            .iter()
            .find(|function| function.name.name == "main");
        let main = main.ok_or_else(|| {
            self.error(
                "program must declare fn main() as its entry point",
                self.typed.program.span,
            )
        })?;
        if !main.parameters.is_empty() {
            return Err(self.error(
                "main entry point cannot have parameters",
                main.parameters[0].span,
            ));
        }
        let main_return = self.function_return_type(main)?;
        if !matches!(main_return, Type::Void | Type::Int) {
            return Err(self.error(
                format!("main must return Void or Int, found {main_return}"),
                main.return_type
                    .as_ref()
                    .map_or(main.name.span, |ty| ty.span),
            ));
        }

        self.output.push_str(C_RUNTIME);
        self.output.push('\n');
        for function in &functions {
            self.emit_function_signature(function, true)?;
            self.output.push_str(";\n");
        }
        self.output.push('\n');
        for function in &functions {
            self.emit_function(function)?;
            self.output.push('\n');
        }

        self.output.push_str("int main(void) {\n");
        if main_return == Type::Int {
            self.output
                .push_str(&format!("    return (int){}();\n", function_name("main")));
        } else {
            self.output.push_str(&format!(
                "    {}();\n    return 0;\n",
                function_name("main")
            ));
        }
        self.output.push_str("}\n");
        Ok(self.output)
    }

    fn supported_functions(&self) -> Result<Vec<&'a FunctionDeclaration>, Diagnostic> {
        let mut functions = Vec::new();
        for declaration in &self.typed.program.declarations {
            match declaration {
                Declaration::Function(function) => functions.push(function),
                Declaration::Class(item) | Declaration::Struct(item) => {
                    return Err(self.error(
                        "native code generation for classes and structs is not part of the procedural v0.1 backend",
                        item.name.span,
                    ));
                }
                Declaration::Import(item) => {
                    let span = item.path.first().map_or(item.span, |name| name.span);
                    return Err(self.error(
                        "imports are not supported by the procedural v0.1 native backend",
                        span,
                    ));
                }
            }
        }
        Ok(functions)
    }

    fn emit_function_signature(
        &mut self,
        function: &FunctionDeclaration,
        prototype: bool,
    ) -> Result<(), Diagnostic> {
        let return_type = self.function_return_type(function)?;
        let c_return = self.c_type(&return_type, function.name.span)?;
        self.output.push_str(&format!(
            "static {c_return} {}(",
            function_name(&function.name.name)
        ));
        if function.parameters.is_empty() {
            self.output.push_str("void");
        } else {
            for (index, parameter) in function.parameters.iter().enumerate() {
                if index > 0 {
                    self.output.push_str(", ");
                }
                let ty = self.typed.parameter_type(parameter).unwrap_or(&Type::Error);
                let c_type = self.c_type(ty, parameter.type_reference.span)?;
                let parameter_type = if *ty == Type::String {
                    c_type.to_owned()
                } else {
                    format!("const {c_type}")
                };
                self.output.push_str(&format!(
                    "{parameter_type} {}",
                    parameter_name(index, &parameter.name.name)
                ));
            }
        }
        self.output.push(')');
        if !prototype {
            self.output.push(' ');
        }
        Ok(())
    }

    fn emit_function(&mut self, function: &FunctionDeclaration) -> Result<(), Diagnostic> {
        let return_type = self.function_return_type(function)?;
        if return_type != Type::Void && !block_returns(&function.body) {
            return Err(self.error(
                format!(
                    "function '{}' may finish without returning {return_type}",
                    function.name.name
                ),
                function.body.span,
            ));
        }

        self.emit_function_signature(function, false)?;
        self.output.push_str("{\n");
        self.push_scope();
        for (index, parameter) in function.parameters.iter().enumerate() {
            self.bind(
                &parameter.name.name,
                parameter_name(index, &parameter.name.name),
            );
        }
        self.emit_block_contents(&function.body, 1)?;
        self.pop_scope();
        self.output.push_str("}\n");
        Ok(())
    }

    fn emit_block_contents(&mut self, block: &Block, indent: usize) -> Result<(), Diagnostic> {
        for statement in &block.statements {
            self.emit_statement(statement, indent)?;
        }
        Ok(())
    }

    fn emit_statement(&mut self, statement: &Statement, indent: usize) -> Result<(), Diagnostic> {
        match &statement.kind {
            StatementKind::Variable(variable) => {
                let initializer = variable
                    .initializer
                    .as_ref()
                    .map(|value| self.emit_expression(value))
                    .transpose()?;
                let ty = self.typed.variable_type(variable).unwrap_or(&Type::Error);
                let c_type = self.c_type(ty, variable.name.span)?;
                let c_name = self.fresh_local(&variable.name.name);
                let declaration_type = match (variable.mutable, ty) {
                    (false, Type::String) => "const char * const".to_owned(),
                    (false, _) => format!("const {c_type}"),
                    (true, _) => c_type.to_owned(),
                };
                if let Some(initializer) = initializer {
                    self.line(
                        indent,
                        &format!("{declaration_type} {c_name} = {initializer};"),
                    );
                } else {
                    self.line(indent, &format!("{c_type} {c_name};"));
                }
                self.bind(&variable.name.name, c_name);
            }
            StatementKind::Assignment(assignment) => {
                let name = self.assignment_name(&assignment.target)?;
                let value = self.emit_expression(&assignment.value)?;
                let target_type = self
                    .typed
                    .expression_type(&assignment.target)
                    .unwrap_or(&Type::Error);
                if assignment.operator == AssignmentOperator::AddAssign
                    && target_type == &Type::String
                {
                    self.line(indent, &format!("{name} = princi_concat({name}, {value});"));
                } else {
                    let operator = match assignment.operator {
                        AssignmentOperator::Assign => "=",
                        AssignmentOperator::AddAssign => "+=",
                        AssignmentOperator::SubtractAssign => "-=",
                        AssignmentOperator::MultiplyAssign => "*=",
                    };
                    self.line(indent, &format!("{name} {operator} {value};"));
                }
            }
            StatementKind::If(if_statement) => {
                let condition = self.emit_expression(&if_statement.condition)?;
                self.line(indent, &format!("if ({condition}) {{"));
                self.push_scope();
                self.emit_block_contents(&if_statement.then_branch, indent + 1)?;
                self.pop_scope();
                self.line(indent, "}");
                if let Some(else_branch) = &if_statement.else_branch {
                    self.line(indent, "else {");
                    self.emit_nested_statement(else_branch, indent + 1)?;
                    self.line(indent, "}");
                }
            }
            StatementKind::While(while_statement) => {
                let condition = self.emit_expression(&while_statement.condition)?;
                self.line(indent, &format!("while ({condition}) {{"));
                self.push_scope();
                self.emit_block_contents(&while_statement.body, indent + 1)?;
                self.pop_scope();
                self.line(indent, "}");
            }
            StatementKind::For(for_statement) => {
                let ExpressionKind::Range { start, end } = ungroup_kind(&for_statement.range.kind)
                else {
                    return Err(self.error(
                        "for loop requires an integer range",
                        for_statement.range.span,
                    ));
                };
                let start = self.emit_expression(start)?;
                let end = self.emit_expression(end)?;
                let c_name = self.fresh_local(&for_statement.variable.name);
                self.line(indent, "{");
                self.line(
                    indent + 1,
                    &format!("const int64_t princi_start_{c_name} = {start};"),
                );
                self.line(
                    indent + 1,
                    &format!("const int64_t princi_end_{c_name} = {end};"),
                );
                self.line(
                    indent + 1,
                    &format!(
                        "for (int64_t princi_index_{c_name} = princi_start_{c_name}; princi_index_{c_name} < princi_end_{c_name}; ++princi_index_{c_name}) {{"
                    ),
                );
                self.push_scope();
                self.bind(&for_statement.variable.name, c_name.clone());
                self.line(
                    indent + 2,
                    &format!("const int64_t {c_name} = princi_index_{c_name};"),
                );
                self.push_scope();
                self.emit_block_contents(&for_statement.body, indent + 2)?;
                self.pop_scope();
                self.pop_scope();
                self.line(indent + 1, "}");
                self.line(indent, "}");
            }
            StatementKind::Return(value) => {
                if let Some(value) = value {
                    let value = self.emit_expression(value)?;
                    self.line(indent, &format!("return {value};"));
                } else {
                    self.line(indent, "return;");
                }
            }
            StatementKind::Expression(expression) => {
                let expression = self.emit_expression(expression)?;
                self.line(indent, &format!("{expression};"));
            }
            StatementKind::Block(block) => self.emit_nested_block(block, indent)?,
        }
        Ok(())
    }

    fn emit_nested_statement(
        &mut self,
        statement: &Statement,
        indent: usize,
    ) -> Result<(), Diagnostic> {
        if let StatementKind::Block(block) = &statement.kind {
            self.push_scope();
            self.emit_block_contents(block, indent)?;
            self.pop_scope();
            Ok(())
        } else {
            self.push_scope();
            self.emit_statement(statement, indent)?;
            self.pop_scope();
            Ok(())
        }
    }

    fn emit_nested_block(&mut self, block: &Block, indent: usize) -> Result<(), Diagnostic> {
        self.line(indent, "{");
        self.push_scope();
        self.emit_block_contents(block, indent + 1)?;
        self.pop_scope();
        self.line(indent, "}");
        Ok(())
    }

    fn emit_expression(&mut self, expression: &Expression) -> Result<String, Diagnostic> {
        match &expression.kind {
            ExpressionKind::Identifier(identifier) => {
                self.lookup(&identifier.name).ok_or_else(|| {
                    self.error(
                        format!(
                            "code generation could not resolve local '{}'",
                            identifier.name
                        ),
                        identifier.span,
                    )
                })
            }
            ExpressionKind::Literal(literal) => match literal {
                Literal::Integer(value) => {
                    let integer = value.parse::<u128>().map_err(|_| {
                        self.error(
                            "integer literal is outside the supported Int range",
                            expression.span,
                        )
                    })?;
                    if integer > i64::MAX as u128 {
                        Err(self.error(
                            "integer literal is outside the supported Int range",
                            expression.span,
                        ))
                    } else {
                        Ok(format!("INT64_C({integer})"))
                    }
                }
                Literal::FloatingPoint(value) => {
                    if !matches!(value.parse::<f64>(), Ok(number) if number.is_finite()) {
                        Err(self.error(
                            "floating-point literal is outside the supported Float range",
                            expression.span,
                        ))
                    } else {
                        Ok(value.clone())
                    }
                }
                Literal::String(value) => Ok(c_string(value)),
                Literal::Boolean(value) => Ok(if *value { "true" } else { "false" }.to_owned()),
            },
            ExpressionKind::Call { callee, arguments } => {
                let ExpressionKind::Identifier(identifier) = ungroup_kind(&callee.kind) else {
                    return Err(self.error(
                        "only direct function calls are supported by the procedural v0.1 backend",
                        callee.span,
                    ));
                };
                if self.lookup(&identifier.name).is_some() {
                    return Err(self.error(
                        "indirect calls through local values are outside the procedural v0.1 backend",
                        callee.span,
                    ));
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.emit_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                if identifier.name == "print" {
                    if arguments.len() != 1 {
                        return Err(
                            self.error("print expects exactly one argument", expression.span)
                        );
                    }
                    let actual_type =
                        call_argument_type(expression, self.typed).ok_or_else(|| {
                            self.error("could not determine print argument type", expression.span)
                        })?;
                    let print_function = match actual_type {
                        Type::Int => "princi_print_int",
                        Type::Float => "princi_print_float",
                        Type::Bool => "princi_print_bool",
                        Type::String => "princi_print_string",
                        other => {
                            return Err(self.error(
                                format!("print cannot lower values of type {other}"),
                                expression.span,
                            ));
                        }
                    };
                    Ok(format!("{print_function}({})", arguments[0]))
                } else {
                    Ok(format!(
                        "{}({})",
                        function_name(&identifier.name),
                        arguments.join(", ")
                    ))
                }
            }
            ExpressionKind::Unary { operator, operand } => {
                if *operator == UnaryOperator::Negative {
                    if let ExpressionKind::Literal(Literal::Integer(value)) = &operand.kind {
                        if value.parse::<u128>().ok() == Some((i64::MAX as u128) + 1) {
                            return Ok("(-INT64_C(9223372036854775807) - INT64_C(1))".to_owned());
                        }
                    }
                }
                let operator = match operator {
                    UnaryOperator::Positive => "+",
                    UnaryOperator::Negative => "-",
                    UnaryOperator::Not => "!",
                };
                Ok(format!("({operator}{})", self.emit_expression(operand)?))
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = self.emit_expression(left)?;
                let right_value = self.emit_expression(right)?;
                let operand_type = self.typed.expression_type(left).unwrap_or(&Type::Error);
                match operator {
                    BinaryOperator::Add if operand_type == &Type::String => {
                        Ok(format!("princi_concat({left_value}, {right_value})"))
                    }
                    BinaryOperator::Equal | BinaryOperator::NotEqual
                        if operand_type == &Type::String =>
                    {
                        let comparison = if *operator == BinaryOperator::Equal {
                            "=="
                        } else {
                            "!="
                        };
                        Ok(format!(
                            "(strcmp({left_value}, {right_value}) {comparison} 0)"
                        ))
                    }
                    _ => {
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
                        Ok(format!("({left_value} {operator} {right_value})"))
                    }
                }
            }
            ExpressionKind::Group(inner) => Ok(format!("({})", self.emit_expression(inner)?)),
            ExpressionKind::SelfValue
            | ExpressionKind::List(_)
            | ExpressionKind::Construction { .. }
            | ExpressionKind::Member { .. }
            | ExpressionKind::Index { .. }
            | ExpressionKind::Range { .. } => Err(self.error(
                "this expression is outside the procedural v0.1 native backend",
                expression.span,
            )),
        }
    }

    fn assignment_name(&self, target: &Expression) -> Result<String, Diagnostic> {
        match ungroup_kind(&target.kind) {
            ExpressionKind::Identifier(identifier) => {
                self.lookup(&identifier.name).ok_or_else(|| {
                    self.error(
                        format!(
                            "code generation could not resolve assignment target '{}'",
                            identifier.name
                        ),
                        identifier.span,
                    )
                })
            }
            _ => Err(self.error(
                "the procedural v0.1 backend supports assignment to local variables only",
                target.span,
            )),
        }
    }

    fn function_return_type(&self, function: &FunctionDeclaration) -> Result<Type, Diagnostic> {
        match self.typed.symbols.global(&function.name.name) {
            Some(GlobalSymbol::Function(signature)) => Ok(*signature.return_type.clone()),
            _ => Err(self.error(
                format!(
                    "missing semantic signature for function '{}'",
                    function.name.name
                ),
                function.name.span,
            )),
        }
    }

    fn c_type(&self, ty: &Type, span: SourceSpan) -> Result<&'static str, Diagnostic> {
        match ty {
            Type::Int => Ok("int64_t"),
            Type::Float => Ok("double"),
            Type::Bool => Ok("bool"),
            Type::String => Ok("const char *"),
            Type::Void => Ok("void"),
            other => Err(self.error(
                format!("native code generation does not support type {other}"),
                span,
            )),
        }
    }

    fn error(&self, message: impl Into<String>, span: SourceSpan) -> Diagnostic {
        Diagnostic::at(message, self.source.location(span))
    }

    fn line(&mut self, indent: usize, text: &str) {
        for _ in 0..indent {
            self.output.push_str("    ");
        }
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind(&mut self, name: &str, c_name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_owned(), c_name);
        }
    }

    fn lookup(&self, name: &str) -> Option<String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn fresh_local(&mut self, name: &str) -> String {
        let id = self.next_local;
        self.next_local += 1;
        format!("princi_local_{id}_{}", encode_identifier(name))
    }
}

#[cfg(test)]
mod tests {
    use super::generate_c;
    use crate::lexer;
    use crate::parser;
    use crate::semantic;
    use crate::source::SourceFile;

    fn lower(text: &str) -> Result<String, String> {
        let source = SourceFile::from_text("backend.prnc", text);
        let tokens = lexer::lex(&source).map_err(|errors| errors.to_string())?;
        let parsed = parser::parse(&source, tokens);
        if !parsed.diagnostics.is_empty() {
            return Err(parsed
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        let analyzed = semantic::analyze(&source, &parsed.program);
        if !analyzed.diagnostics.is_empty() {
            return Err(analyzed
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        generate_c(&analyzed.typed_program, &source).map_err(|error| error.to_string())
    }

    #[test]
    fn lowers_recursive_functions_and_procedural_control_flow() {
        let c = lower(
            r#"fn factorial(n: Int) -> Int {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}
fn main() {
    var value = factorial(5)
    for index in 0..3 { value += index }
    while value < 123 { value += 1 }
    if value == 123 { print(value) } else { print(0) }
}"#,
        )
        .expect("procedural source should lower");

        assert!(c.contains("static int64_t princi_fn_666163746f7269616c(const int64_t"));
        assert!(c.contains("princi_fn_666163746f7269616c((princi_param_0_6e"));
        assert!(c.contains("for (int64_t princi_index_"));
        assert!(c.contains("while ((princi_local_"));
        assert!(c.contains("if ((princi_local_"));
        assert!(c.contains("int main(void)"));
    }

    #[test]
    fn lowers_type_directed_strings_floats_booleans_and_print_calls() {
        let c = lower(
            r#"fn main() {
    let greeting: String = "Hello, " + "Princi"
    let equal = greeting == "Hello, Princi"
    let measurement: Float = 1.5 * 2.0
    let ready: Bool = equal && measurement >= 3.0
    print(greeting)
    print(measurement)
    print(ready)
}"#,
        )
        .expect("primitive source should lower");

        assert!(c.contains("princi_concat("));
        assert!(c.contains("strcmp("));
        assert!(c.contains("double princi_local_"));
        assert!(c.contains("princi_print_string("));
        assert!(c.contains("princi_print_float("));
        assert!(c.contains("princi_print_bool("));
        assert!(c.contains("&&"));
    }

    #[test]
    fn reports_unsupported_backend_syntax_at_its_source_location() {
        let error = lower("struct Point {}\nfn main() {}")
            .expect_err("class and struct code generation is outside the subset");
        assert!(error.contains("backend.prnc:1:8: error:"));
        assert!(error.contains("classes and structs"));
    }
}

fn function_name(name: &str) -> String {
    format!("princi_fn_{}", encode_identifier(name))
}

fn parameter_name(index: usize, name: &str) -> String {
    format!("princi_param_{index}_{}", encode_identifier(name))
}

fn encode_identifier(name: &str) -> String {
    name.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn c_string(value: &str) -> String {
    let mut output = String::from("\"");
    for byte in value.as_bytes() {
        match byte {
            b'"' => output.push_str("\\\""),
            b'\\' => output.push_str("\\\\"),
            b'\n' => output.push_str("\\n"),
            b'\r' => output.push_str("\\r"),
            b'\t' => output.push_str("\\t"),
            0x20..=0x7e => output.push(*byte as char),
            _ => output.push_str(&format!("\\{byte:03o}")),
        }
    }
    output.push('"');
    output
}

fn call_argument_type<'a>(expression: &Expression, typed: &'a TypedProgram) -> Option<&'a Type> {
    let ExpressionKind::Call { arguments, .. } = &expression.kind else {
        return None;
    };
    arguments
        .first()
        .and_then(|argument| typed.expression_type(argument))
}

fn block_returns(block: &Block) -> bool {
    block.statements.iter().any(statement_returns)
}

fn statement_returns(statement: &Statement) -> bool {
    match &statement.kind {
        StatementKind::Return(_) => true,
        StatementKind::Block(block) => block_returns(block),
        StatementKind::If(if_statement) => {
            if_statement
                .else_branch
                .as_deref()
                .is_some_and(statement_returns)
                && block_returns(&if_statement.then_branch)
        }
        _ => false,
    }
}

fn ungroup_kind(mut kind: &ExpressionKind) -> &ExpressionKind {
    while let ExpressionKind::Group(inner) = kind {
        kind = &inner.kind;
    }
    kind
}
