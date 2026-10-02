use std::collections::{HashMap, HashSet};

use crate::ast::{
    Assignment, AssignmentOperator, BinaryOperator, Block, ClassMember, Declaration, Expression,
    ExpressionKind, ExternBlockDeclaration, FieldDeclaration, FunctionDeclaration, Identifier,
    InitializerDeclaration, Literal, Program, Statement, StatementKind, TypeDeclaration,
    TypeDeclarationKind, UnaryOperator, VariableDeclaration,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::ffi::{CAbiType, ExternalFunctionType};
use crate::modules::ResolvedModules;
use crate::source::{SourceFile, SourceSpan};
use crate::types::{FunctionType, Type};

#[derive(Debug, Clone)]
pub struct TypedProgram {
    pub program: Program,
    pub symbols: SymbolTable,
    pub modules: ResolvedModules,
    expression_types: HashMap<SourceSpan, Type>,
    variable_types: HashMap<SourceSpan, Type>,
}

impl TypedProgram {
    pub fn expression_type(&self, expression: &Expression) -> Option<&Type> {
        self.expression_types.get(&expression.span)
    }

    pub fn variable_type(&self, variable: &VariableDeclaration) -> Option<&Type> {
        self.variable_types.get(&variable.name.span)
    }

    pub fn parameter_type(&self, parameter: &crate::ast::Parameter) -> Option<&Type> {
        self.variable_types.get(&parameter.name.span)
    }

    pub fn binding_type(&self, identifier: &Identifier) -> Option<&Type> {
        self.variable_types.get(&identifier.span)
    }
}

#[derive(Debug, Clone)]
pub struct SemanticResult {
    pub typed_program: TypedProgram,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn analyze(source: &SourceFile, program: &Program) -> SemanticResult {
    analyze_with_modules(source, program, &ResolvedModules::default())
}

pub fn analyze_with_modules(
    source: &SourceFile,
    program: &Program,
    modules: &ResolvedModules,
) -> SemanticResult {
    Analyzer::new(source, modules).run(program)
}

#[derive(Debug, Clone)]
struct VariableSymbol {
    ty: Type,
    mutable: bool,
    initialized: bool,
}

#[derive(Debug, Clone)]
pub enum GlobalSymbol {
    Function(FunctionType),
    ExternalFunction(ExternalFunctionType),
    Type(Type),
}

#[derive(Debug, Clone)]
pub struct TypeSymbols {
    pub ty: Type,
    pub fields: HashMap<String, Type>,
    pub field_order: Vec<String>,
    pub methods: HashMap<String, FunctionType>,
    pub initializer: Option<FunctionType>,
    member_names: HashSet<String>,
}

impl TypeSymbols {
    fn new(ty: Type) -> Self {
        Self {
            ty,
            fields: HashMap::new(),
            field_order: Vec::new(),
            methods: HashMap::new(),
            initializer: None,
            member_names: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    globals: HashMap<String, GlobalSymbol>,
    types: HashMap<String, TypeSymbols>,
}

impl SymbolTable {
    pub fn global(&self, name: &str) -> Option<&GlobalSymbol> {
        self.globals.get(name)
    }

    pub fn type_symbols(&self, name: &str) -> Option<&TypeSymbols> {
        self.types.get(name)
    }
}

#[derive(Debug, Clone, Default)]
struct Scope {
    variables: HashMap<String, VariableSymbol>,
}

struct Analyzer<'a> {
    source: &'a SourceFile,
    modules: ResolvedModules,
    globals: HashMap<String, GlobalSymbol>,
    types: HashMap<String, TypeSymbols>,
    scopes: Vec<Scope>,
    accepted_type_declarations: HashSet<SourceSpan>,
    accepted_functions: HashSet<SourceSpan>,
    accepted_methods: HashSet<SourceSpan>,
    accepted_initializers: HashSet<SourceSpan>,
    function_signatures: HashMap<SourceSpan, FunctionType>,
    initializer_signatures: HashMap<SourceSpan, FunctionType>,
    current_return_type: Option<Type>,
    current_type: Option<Type>,
    expression_types: HashMap<SourceSpan, Type>,
    variable_types: HashMap<SourceSpan, Type>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Analyzer<'a> {
    fn new(source: &'a SourceFile, modules: &ResolvedModules) -> Self {
        let print_signature = FunctionType::new(vec![Type::Any], Type::Void);
        Self {
            source,
            modules: modules.clone(),
            globals: HashMap::from([
                (
                    "print".to_owned(),
                    GlobalSymbol::Function(print_signature.clone()),
                ),
                (
                    "println".to_owned(),
                    GlobalSymbol::Function(print_signature),
                ),
            ]),
            types: HashMap::new(),
            scopes: Vec::new(),
            accepted_type_declarations: HashSet::new(),
            accepted_functions: HashSet::new(),
            accepted_methods: HashSet::new(),
            accepted_initializers: HashSet::new(),
            function_signatures: HashMap::new(),
            initializer_signatures: HashMap::new(),
            current_return_type: None,
            current_type: None,
            expression_types: HashMap::new(),
            variable_types: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self, program: &Program) -> SemanticResult {
        self.register_global_names(program);
        self.resolve_global_functions(program);
        self.register_type_members(program);
        self.validate_struct_value_layouts(program);
        self.check_declaration_bodies(program);

        SemanticResult {
            typed_program: TypedProgram {
                program: program.clone(),
                symbols: SymbolTable {
                    globals: self.globals,
                    types: self.types,
                },
                modules: self.modules.clone(),
                expression_types: self.expression_types,
                variable_types: self.variable_types,
            },
            diagnostics: self.diagnostics,
        }
    }

    fn register_global_names(&mut self, program: &Program) {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Class(type_declaration) | Declaration::Struct(type_declaration) => {
                    let name = &type_declaration.name.name;
                    if is_reserved_type_name(name) || self.globals.contains_key(name) {
                        self.error_with(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("duplicate or reserved declaration '{name}'"),
                            type_declaration.name.span,
                        );
                        continue;
                    }

                    let ty = match type_declaration.kind {
                        TypeDeclarationKind::Class => Type::Class(name.clone()),
                        TypeDeclarationKind::Struct => Type::Struct(name.clone()),
                    };
                    self.globals
                        .insert(name.clone(), GlobalSymbol::Type(ty.clone()));
                    self.types.insert(name.clone(), TypeSymbols::new(ty));
                    self.accepted_type_declarations
                        .insert(type_declaration.span);
                }
                Declaration::Function(function) => {
                    let name = &function.name.name;
                    if self.globals.contains_key(name) || is_reserved_type_name(name) {
                        self.error_with(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("duplicate declaration '{name}'"),
                            function.name.span,
                        );
                        continue;
                    }
                    self.globals.insert(
                        name.clone(),
                        GlobalSymbol::Function(FunctionType::new(Vec::new(), Type::Error)),
                    );
                    self.accepted_functions.insert(function.span);
                }
                Declaration::ExternBlock(block) => self.register_external_block(block),
                Declaration::Import(_) => {}
            }
        }
    }

    fn register_external_block(&mut self, block: &ExternBlockDeclaration) {
        if block.abi != "C" {
            self.error(
                format!(
                    "unsupported extern ABI '{}'; only extern \"C\" is supported in v0.1",
                    block.abi
                ),
                block.abi_span,
            );
            return;
        }

        for function in &block.functions {
            let name = &function.name.name;
            if !is_ascii_c_identifier(name) {
                self.error(
                    "C external function names must be ASCII C identifiers",
                    function.name.span,
                );
                continue;
            }
            if is_reserved_external_symbol(name) {
                self.error(
                    format!("external symbol '{name}' conflicts with a compiler/runtime symbol"),
                    function.name.span,
                );
                continue;
            }
            if self.globals.contains_key(name) || is_reserved_type_name(name) {
                self.error_with(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate or reserved declaration '{name}'"),
                    function.name.span,
                );
                continue;
            }

            let mut parameter_types = Vec::with_capacity(function.parameters.len());
            let mut valid = true;
            for parameter in &function.parameters {
                match self.resolve_c_abi_type(&parameter.type_reference, false) {
                    Some(ty) => parameter_types.push(ty),
                    None => valid = false,
                }
            }
            let return_type = match function.return_type.as_ref() {
                Some(type_reference) => self.resolve_c_abi_type(type_reference, true),
                None => Some(CAbiType::Void),
            };
            let Some(return_type) = return_type else {
                continue;
            };

            if valid {
                self.globals.insert(
                    name.clone(),
                    GlobalSymbol::ExternalFunction(ExternalFunctionType::new(
                        parameter_types,
                        return_type,
                    )),
                );
            }
        }
    }

    fn resolve_c_abi_type(
        &mut self,
        type_reference: &crate::ast::TypeReference,
        allow_void: bool,
    ) -> Option<CAbiType> {
        let ty = match type_reference.name.as_str() {
            "Int32" => CAbiType::Int32,
            "Int64" => CAbiType::Int64,
            "Float64" => CAbiType::Float64,
            "Void" if allow_void => CAbiType::Void,
            "Void" => {
                self.error(
                    "Void is only valid as an extern function return type",
                    type_reference.span,
                );
                return None;
            }
            name => {
                self.error(
                    format!(
                        "type '{name}' is not FFI-safe in v0.1; use Int32, Int64, Float64{}",
                        if allow_void { ", or Void" } else { "" }
                    ),
                    type_reference.span,
                );
                return None;
            }
        };
        if !type_reference.arguments.is_empty() {
            self.error(
                "generic types are not supported in extern \"C\" declarations",
                type_reference.span,
            );
            return None;
        }
        Some(ty)
    }

    fn resolve_global_functions(&mut self, program: &Program) {
        for declaration in &program.declarations {
            let Declaration::Function(function) = declaration else {
                continue;
            };
            if !self.accepted_functions.contains(&function.span) {
                continue;
            }

            let signature =
                self.function_signature(&function.parameters, function.return_type.as_ref());
            self.globals.insert(
                function.name.name.clone(),
                GlobalSymbol::Function(signature.clone()),
            );
            self.function_signatures.insert(function.span, signature);
        }
    }

    fn register_type_members(&mut self, program: &Program) {
        for declaration in &program.declarations {
            let (Declaration::Class(type_declaration) | Declaration::Struct(type_declaration)) =
                declaration
            else {
                continue;
            };
            if !self
                .accepted_type_declarations
                .contains(&type_declaration.span)
            {
                continue;
            }

            for member in &type_declaration.members {
                match member {
                    ClassMember::Field(field) => self.register_field(type_declaration, field),
                    ClassMember::Method(method)
                        if type_declaration.kind == TypeDeclarationKind::Struct =>
                    {
                        self.error("methods on structs are not supported in v0.1", method.name.span)
                    }
                    ClassMember::Method(method) => self.register_method(type_declaration, method),
                    ClassMember::Initializer(initializer)
                        if type_declaration.kind == TypeDeclarationKind::Struct =>
                    {
                        self.error(
                            "structs do not support init constructors in v0.1; construct them from fields",
                            initializer.span,
                        )
                    }
                    ClassMember::Initializer(initializer) => {
                        self.register_initializer(type_declaration, initializer)
                    }
                }
            }
        }
    }

    fn validate_struct_value_layouts(&mut self, program: &Program) {
        let mut graph = HashMap::<String, Vec<(String, SourceSpan)>>::new();
        let mut source_order = Vec::new();
        for declaration in &program.declarations {
            let Declaration::Struct(structure) = declaration else {
                continue;
            };
            if !self.accepted_type_declarations.contains(&structure.span) {
                continue;
            }
            let mut fields = Vec::new();
            for member in &structure.members {
                let ClassMember::Field(field) = member else {
                    continue;
                };
                if let Some(Type::Struct(target)) = self
                    .types
                    .get(&structure.name.name)
                    .and_then(|info| info.fields.get(&field.name.name))
                {
                    fields.push((target.clone(), field.type_reference.span));
                }
            }
            graph.insert(structure.name.name.clone(), fields);
            source_order.push(structure.name.name.clone());
        }

        fn find_cycle(
            name: &str,
            graph: &HashMap<String, Vec<(String, SourceSpan)>>,
            states: &mut HashMap<String, u8>,
        ) -> Option<SourceSpan> {
            states.insert(name.to_owned(), 1);
            for (target, span) in graph.get(name).into_iter().flatten() {
                match states.get(target).copied().unwrap_or(0) {
                    1 => return Some(*span),
                    2 => continue,
                    _ => {
                        if let Some(cycle) = find_cycle(target, graph, states) {
                            return Some(cycle);
                        }
                    }
                }
            }
            states.insert(name.to_owned(), 2);
            None
        }

        let mut states = HashMap::new();
        for name in source_order {
            if states.get(&name).copied().unwrap_or(0) == 0 {
                if let Some(span) = find_cycle(&name, &graph, &mut states) {
                    self.error(
                        "recursive struct fields do not have a finite value layout",
                        span,
                    );
                    break;
                }
            }
        }
    }

    fn register_field(&mut self, owner: &TypeDeclaration, field: &FieldDeclaration) {
        if !self.claim_member_name(owner, &field.name) {
            return;
        }
        let field_type = self.resolve_type_reference(&field.type_reference, false);
        if let Some(info) = self.types.get_mut(&owner.name.name) {
            info.field_order.push(field.name.name.clone());
            info.fields.insert(field.name.name.clone(), field_type);
        }
    }

    fn register_method(&mut self, owner: &TypeDeclaration, method: &FunctionDeclaration) {
        if !self.claim_member_name(owner, &method.name) {
            return;
        }
        let signature = self.function_signature(&method.parameters, method.return_type.as_ref());
        if let Some(info) = self.types.get_mut(&owner.name.name) {
            info.methods
                .insert(method.name.name.clone(), signature.clone());
        }
        self.function_signatures.insert(method.span, signature);
        self.accepted_methods.insert(method.span);
    }

    fn register_initializer(
        &mut self,
        owner: &TypeDeclaration,
        initializer: &InitializerDeclaration,
    ) {
        let already_has_initializer = self
            .types
            .get(&owner.name.name)
            .is_some_and(|info| info.initializer.is_some());
        if already_has_initializer {
            self.error_with(
                DiagnosticCode::DuplicateDeclaration,
                "duplicate init constructor",
                initializer.span,
            );
            return;
        }

        let signature = self.function_signature(&initializer.parameters, None);
        if let Some(info) = self.types.get_mut(&owner.name.name) {
            info.initializer = Some(signature.clone());
        }
        self.initializer_signatures
            .insert(initializer.span, signature);
        self.accepted_initializers.insert(initializer.span);
    }

    fn claim_member_name(&mut self, owner: &TypeDeclaration, name: &Identifier) -> bool {
        let Some(info) = self.types.get_mut(&owner.name.name) else {
            return false;
        };
        if !info.member_names.insert(name.name.clone()) {
            self.error_with(
                DiagnosticCode::DuplicateDeclaration,
                format!(
                    "duplicate member '{}' in type '{}'",
                    name.name, owner.name.name
                ),
                name.span,
            );
            return false;
        }
        true
    }

    fn function_signature(
        &mut self,
        parameters: &[crate::ast::Parameter],
        return_type: Option<&crate::ast::TypeReference>,
    ) -> FunctionType {
        let parameters = parameters
            .iter()
            .map(|parameter| self.resolve_type_reference(&parameter.type_reference, false))
            .collect();
        let return_type = return_type
            .map(|type_reference| self.resolve_type_reference(type_reference, true))
            .unwrap_or(Type::Void);
        FunctionType::new(parameters, return_type)
    }

    fn resolve_type_reference(
        &mut self,
        type_reference: &crate::ast::TypeReference,
        allow_void: bool,
    ) -> Type {
        let ty = match type_reference.name.as_str() {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "String" => Type::String,
            "Void" => Type::Void,
            "Int32" | "Int64" | "Float64" => {
                self.error(
                    "C ABI types may only be used in extern \"C\" declarations",
                    type_reference.span,
                );
                Type::Error
            }
            "List" => {
                if type_reference.arguments.len() != 1 {
                    self.error(
                        "List requires exactly one type argument, such as List<Int>",
                        type_reference.span,
                    );
                    Type::Error
                } else {
                    let element = self.resolve_type_reference(&type_reference.arguments[0], false);
                    if element.is_error() || element == Type::Void {
                        Type::Error
                    } else {
                        Type::List(Box::new(element))
                    }
                }
            }
            _ if !type_reference.arguments.is_empty() => {
                self.error(
                    "generic type arguments are only supported for List<T> in v0.1",
                    type_reference.span,
                );
                Type::Error
            }
            name => self
                .types
                .get(name)
                .map(|info| info.ty.clone())
                .unwrap_or_else(|| {
                    self.error_with(
                        DiagnosticCode::UnknownType,
                        format!("unknown type '{}'", type_reference.name),
                        type_reference.span,
                    );
                    Type::Error
                }),
        };

        if matches!(ty, Type::Void) && !allow_void {
            self.error(
                "Void is only valid as a function return type",
                type_reference.span,
            );
            Type::Error
        } else {
            ty
        }
    }

    fn check_declaration_bodies(&mut self, program: &Program) {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(function)
                    if self.accepted_functions.contains(&function.span) =>
                {
                    let signature = self.function_signatures[&function.span].clone();
                    self.check_routine(&function.parameters, &function.body, &signature, None);
                }
                Declaration::Class(type_declaration) | Declaration::Struct(type_declaration)
                    if self
                        .accepted_type_declarations
                        .contains(&type_declaration.span) =>
                {
                    self.check_type_bodies(type_declaration);
                }
                _ => {}
            }
        }
    }

    fn check_type_bodies(&mut self, type_declaration: &TypeDeclaration) {
        let Some(owner_type) = self
            .types
            .get(&type_declaration.name.name)
            .map(|info| info.ty.clone())
        else {
            return;
        };

        for member in &type_declaration.members {
            match member {
                ClassMember::Method(method) if self.accepted_methods.contains(&method.span) => {
                    let signature = self.function_signatures[&method.span].clone();
                    self.check_routine(
                        &method.parameters,
                        &method.body,
                        &signature,
                        Some(owner_type.clone()),
                    );
                }
                ClassMember::Initializer(initializer)
                    if self.accepted_initializers.contains(&initializer.span) =>
                {
                    let signature = self.initializer_signatures[&initializer.span].clone();
                    self.check_routine(
                        &initializer.parameters,
                        &initializer.body,
                        &signature,
                        Some(owner_type.clone()),
                    );
                }
                _ => {}
            }
        }
    }

    fn check_routine(
        &mut self,
        parameters: &[crate::ast::Parameter],
        body: &Block,
        signature: &FunctionType,
        owner: Option<Type>,
    ) {
        let previous_return = self
            .current_return_type
            .replace(*signature.return_type.clone());
        let previous_type = std::mem::replace(&mut self.current_type, owner.clone());
        self.push_scope();

        if let Some(owner_type) = owner {
            self.declare_local(
                "self",
                &Identifier {
                    name: "self".to_owned(),
                    span: body.span,
                },
                owner_type,
                true,
                true,
            );
        }

        for (index, parameter) in parameters.iter().enumerate() {
            let ty = signature
                .parameters
                .get(index)
                .cloned()
                .unwrap_or(Type::Error);
            self.variable_types.insert(parameter.name.span, ty.clone());
            self.declare_local(&parameter.name.name, &parameter.name, ty, false, true);
        }

        self.check_block(body, false);
        self.pop_scope();
        self.current_return_type = previous_return;
        self.current_type = previous_type;
    }

    fn check_block(&mut self, block: &Block, introduce_scope: bool) {
        if introduce_scope {
            self.push_scope();
        }
        for statement in &block.statements {
            self.check_statement(statement);
        }
        if introduce_scope {
            self.pop_scope();
        }
    }

    fn check_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Variable(variable) => self.check_variable(variable),
            StatementKind::Assignment(assignment) => self.check_assignment(assignment),
            StatementKind::If(if_statement) => {
                let condition = self.check_expression(&if_statement.condition, None);
                self.require_type(
                    &Type::Bool,
                    &condition,
                    if_statement.condition.span,
                    "if condition",
                );
                let before = self.scopes.clone();
                self.check_block(&if_statement.then_branch, true);
                let then_scopes = self.scopes.clone();
                self.scopes = before.clone();
                if let Some(else_branch) = &if_statement.else_branch {
                    self.check_statement(else_branch);
                }
                for (scope_index, scope) in self.scopes.iter_mut().enumerate() {
                    for (name, variable) in &mut scope.variables {
                        let initialized_in_then = then_scopes
                            .get(scope_index)
                            .and_then(|then_scope| then_scope.variables.get(name))
                            .is_some_and(|then_variable| then_variable.initialized);
                        variable.initialized &= initialized_in_then;
                    }
                }
            }
            StatementKind::While(while_statement) => {
                let condition = self.check_expression(&while_statement.condition, None);
                self.require_type(
                    &Type::Bool,
                    &condition,
                    while_statement.condition.span,
                    "while condition",
                );
                let before = self.scopes.clone();
                self.check_block(&while_statement.body, true);
                // A while body may execute zero times, so it cannot establish
                // definite initialization after the loop.
                self.scopes = before;
            }
            StatementKind::For(for_statement) => {
                let range_type = self.check_expression(&for_statement.range, None);
                let element_type = match range_type {
                    Type::Range(element) => *element,
                    Type::Error => Type::Error,
                    other => {
                        self.error(
                            format!("for loop requires a range, found {other}"),
                            for_statement.range.span,
                        );
                        Type::Error
                    }
                };
                if !element_type.is_error() && element_type != Type::Int {
                    self.error(
                        format!("for loop range requires Int bounds, found {element_type}"),
                        for_statement.range.span,
                    );
                }
                let before = self.scopes.clone();
                self.push_scope();
                self.variable_types
                    .insert(for_statement.variable.span, element_type.clone());
                self.declare_local(
                    &for_statement.variable.name,
                    &for_statement.variable,
                    element_type,
                    false,
                    true,
                );
                self.check_block(&for_statement.body, true);
                self.pop_scope();
                // A range may be empty, so assignments inside it are not
                // guaranteed to execute.
                self.scopes = before;
            }
            StatementKind::Return(value) => self.check_return(value.as_ref(), statement.span),
            StatementKind::Expression(expression) => {
                self.check_expression(expression, None);
            }
            StatementKind::Block(block) => self.check_block(block, true),
        }
    }

    fn check_variable(&mut self, variable: &VariableDeclaration) {
        if !variable.mutable && variable.initializer.is_none() {
            self.error(
                "let declaration requires an initializer",
                variable.name.span,
            );
        }
        let declared_type = variable
            .type_reference
            .as_ref()
            .map(|reference| self.resolve_type_reference(reference, false));
        let initializer_type = variable
            .initializer
            .as_ref()
            .map(|initializer| self.check_expression(initializer, declared_type.as_ref()));

        let ty = match (declared_type, initializer_type) {
            (Some(declared), Some(_)) => declared,
            (Some(declared), None) => declared,
            (None, Some(inferred)) if !matches!(inferred, Type::Void) => inferred,
            (None, Some(_)) => {
                self.error(
                    "cannot infer a variable type from a Void expression",
                    variable.name.span,
                );
                Type::Error
            }
            (None, None) => Type::Error,
        };

        self.variable_types.insert(variable.name.span, ty.clone());
        self.declare_local(
            &variable.name.name,
            &variable.name,
            ty,
            variable.mutable,
            variable.initializer.is_some(),
        );
    }

    fn check_assignment(&mut self, assignment: &Assignment) {
        if assignment.operator != AssignmentOperator::Assign {
            if let ExpressionKind::Identifier(identifier) = ungroup_kind(&assignment.target.kind) {
                if self
                    .lookup_variable(&identifier.name)
                    .is_some_and(|variable| !variable.initialized)
                {
                    self.error_with(
                        DiagnosticCode::UninitializedVariable,
                        format!(
                            "variable '{}' may be read before it is initialized",
                            identifier.name
                        ),
                        identifier.span,
                    );
                }
            }
        }
        let (target_type, mutable) = self.check_assignment_target(&assignment.target);
        if !mutable && !target_type.is_error() {
            self.error(
                "cannot assign through an immutable binding",
                assignment.target.span,
            );
        }

        let value_type = match assignment.operator {
            AssignmentOperator::Assign => {
                let expected = (matches!(target_type, Type::List(_))
                    && is_list_literal(&assignment.value))
                .then_some(&target_type);
                self.check_expression(&assignment.value, expected)
            }
            operator => {
                let right = self.check_expression(&assignment.value, None);
                let binary_operator = match operator {
                    AssignmentOperator::AddAssign => Some(BinaryOperator::Add),
                    AssignmentOperator::SubtractAssign => Some(BinaryOperator::Subtract),
                    AssignmentOperator::MultiplyAssign => Some(BinaryOperator::Multiply),
                    AssignmentOperator::Assign => None,
                };
                if let Some(binary_operator) = binary_operator {
                    self.binary_result(
                        binary_operator,
                        &target_type,
                        &right,
                        assignment.target.span,
                    )
                } else {
                    self.error(
                        "invalid compound assignment operator",
                        assignment.target.span,
                    );
                    Type::Error
                }
            }
        };

        if !target_type.accepts(&value_type) {
            self.error_with(
                DiagnosticCode::TypeMismatch,
                format!("expected {target_type}, found {value_type}"),
                assignment.value.span,
            );
        }
        if assignment.operator == AssignmentOperator::Assign {
            self.mark_assignment_initialized(&assignment.target);
        }
    }

    fn check_assignment_target(&mut self, target: &Expression) -> (Type, bool) {
        let result = match &target.kind {
            ExpressionKind::Identifier(identifier) => {
                if let Some(variable) = self.lookup_variable(&identifier.name).cloned() {
                    (variable.ty, variable.mutable)
                } else if self.globals.contains_key(&identifier.name) {
                    self.error_with(
                        DiagnosticCode::InvalidMember,
                        format!("'{}' is not a mutable variable", identifier.name),
                        identifier.span,
                    );
                    (Type::Error, false)
                } else {
                    self.error_with(
                        DiagnosticCode::UnknownIdentifier,
                        format!("undefined identifier '{}'", identifier.name),
                        identifier.span,
                    );
                    (Type::Error, false)
                }
            }
            ExpressionKind::Member { object, member } => {
                let object_type = self.check_expression(object, None);
                if matches!(object_type, Type::List(_)) && member.name == "length" {
                    self.error("list.length is read-only", member.span);
                    return (Type::Error, false);
                }
                if matches!(object_type, Type::Struct(_))
                    && self.is_struct_list_element_path(object)
                {
                    self.error(
                        "mutating fields of struct list elements is not supported in v0.1; assign the updated struct back to the list",
                        target.span,
                    );
                    return (Type::Error, false);
                }
                let (member_type, is_field) = self.member_type(&object_type, member);
                if !is_field && !member_type.is_error() {
                    self.error_with(
                        DiagnosticCode::InvalidMember,
                        format!("method '{}' is not an assignable field", member.name),
                        member.span,
                    );
                    return (Type::Error, false);
                }
                (member_type, self.is_mutable_base(object))
            }
            ExpressionKind::Index { object, index } => {
                let object_type = self.check_expression(object, None);
                let index_type = self.check_expression(index, None);
                self.require_type(&Type::Int, &index_type, index.span, "list index");
                match object_type {
                    // Lists are heap-backed reference values: their elements
                    // remain mutable through a `let` binding.
                    Type::List(element) => (*element, true),
                    Type::Error => (Type::Error, false),
                    other => {
                        self.error(
                            format!("cannot assign through an index on {other}"),
                            target.span,
                        );
                        (Type::Error, false)
                    }
                }
            }
            ExpressionKind::Group(inner) => self.check_assignment_target(inner),
            _ => {
                self.error("invalid assignment target", target.span);
                (Type::Error, false)
            }
        };
        self.expression_types.insert(target.span, result.0.clone());
        result
    }

    fn is_mutable_base(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Identifier(identifier) => self
                .lookup_variable(&identifier.name)
                .is_some_and(|symbol| symbol.mutable),
            ExpressionKind::SelfValue => self.current_type.is_some(),
            ExpressionKind::Member { object, .. } => self.is_mutable_base(object),
            ExpressionKind::Index { object, .. } => {
                matches!(
                    self.expression_types.get(&expression.span),
                    Some(Type::List(_))
                ) || self.is_mutable_base(object)
            }
            ExpressionKind::Group(inner) => self.is_mutable_base(inner),
            _ => false,
        }
    }

    fn check_return(&mut self, value: Option<&Expression>, span: SourceSpan) {
        let Some(return_type) = self.current_return_type.clone() else {
            self.error_with(
                DiagnosticCode::InvalidReturn,
                "return statement is not inside a function",
                span,
            );
            return;
        };

        match value {
            Some(expression) => {
                let expected = (matches!(return_type, Type::List(_))
                    && is_list_literal(expression))
                .then_some(&return_type);
                let actual = self.check_expression(expression, expected);
                if matches!(return_type, Type::Void) {
                    self.error_with(
                        DiagnosticCode::InvalidReturn,
                        "Void function cannot return a value",
                        expression.span,
                    );
                } else {
                    self.require_type(&return_type, &actual, expression.span, "return value");
                }
            }
            None if !matches!(return_type, Type::Void) => {
                self.error_with(
                    DiagnosticCode::InvalidReturn,
                    format!("return statement requires a value of type {return_type}"),
                    span,
                );
            }
            None => {}
        }
    }

    fn check_expression(&mut self, expression: &Expression, expected: Option<&Type>) -> Type {
        let actual = match &expression.kind {
            ExpressionKind::Identifier(identifier) => self.check_identifier(identifier),
            ExpressionKind::SelfValue => self.current_type.clone().unwrap_or_else(|| {
                self.error(
                    "self is only available inside a class or struct method",
                    expression.span,
                );
                Type::Error
            }),
            ExpressionKind::Literal(literal) => match literal {
                Literal::Integer(_) => Type::Int,
                Literal::FloatingPoint(_) => Type::Float,
                Literal::String(_) => Type::String,
                Literal::Boolean(_) => Type::Bool,
            },
            ExpressionKind::List(elements) => self.check_list(elements, expected, expression.span),
            ExpressionKind::Construction { type_name, fields } => {
                self.check_construction(type_name, fields)
            }
            ExpressionKind::Call { callee, arguments } => {
                self.check_call(callee, arguments, expression.span)
            }
            ExpressionKind::Member { object, member } => {
                let object_type = self.check_expression(object, None);
                self.member_type(&object_type, member).0
            }
            ExpressionKind::Index { object, index } => {
                let object_type = self.check_expression(object, None);
                let index_type = self.check_expression(index, None);
                self.require_type(&Type::Int, &index_type, index.span, "list index");
                match object_type {
                    Type::List(element) => *element,
                    Type::Error => Type::Error,
                    other => {
                        self.error(format!("cannot index a value of type {other}"), object.span);
                        Type::Error
                    }
                }
            }
            ExpressionKind::Unary { operator, operand } => {
                let operand_type = self.check_expression(operand, None);
                self.unary_result(*operator, &operand_type, expression.span)
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = self.check_expression(left, None);
                let right_type = self.check_expression(right, None);
                self.binary_result(*operator, &left_type, &right_type, expression.span)
            }
            ExpressionKind::Range { start, end } => {
                let start_type = self.check_expression(start, None);
                let end_type = self.check_expression(end, None);
                if start_type.is_error() || end_type.is_error() {
                    Type::Error
                } else if start_type == end_type && start_type.is_numeric() {
                    Type::Range(Box::new(start_type))
                } else {
                    self.error(
                        format!(
                            "range bounds must have the same numeric type, found {start_type} and {end_type}"
                        ),
                        expression.span,
                    );
                    Type::Error
                }
            }
            ExpressionKind::Group(inner) => {
                let actual = self.check_expression(inner, expected);
                self.expression_types
                    .insert(expression.span, actual.clone());
                return actual;
            }
        };

        if let Some(expected) = expected {
            self.require_type(expected, &actual, expression.span, "expression");
        }
        self.expression_types
            .insert(expression.span, actual.clone());
        actual
    }

    fn check_identifier(&mut self, identifier: &Identifier) -> Type {
        if let Some(variable) = self.lookup_variable(&identifier.name).cloned() {
            if !variable.initialized {
                self.error_with(
                    DiagnosticCode::UninitializedVariable,
                    format!(
                        "variable '{}' may be read before it is initialized",
                        identifier.name
                    ),
                    identifier.span,
                );
                return Type::Error;
            }
            return variable.ty;
        }
        match self.globals.get(&identifier.name) {
            Some(GlobalSymbol::Function(signature)) => Type::Function(signature.clone()),
            Some(GlobalSymbol::ExternalFunction(_)) => {
                self.error(
                    "external C functions must be called directly in v0.1",
                    identifier.span,
                );
                Type::Error
            }
            Some(GlobalSymbol::Type(_)) => {
                self.error(
                    format!("type '{}' cannot be used as a value", identifier.name),
                    identifier.span,
                );
                Type::Error
            }
            None => {
                self.error_with(
                    DiagnosticCode::UnknownIdentifier,
                    format!("undefined identifier '{}'", identifier.name),
                    identifier.span,
                );
                Type::Error
            }
        }
    }

    fn check_list(
        &mut self,
        elements: &[Expression],
        expected: Option<&Type>,
        span: SourceSpan,
    ) -> Type {
        let expected_list_element = match expected {
            Some(Type::List(element)) => Some(element.as_ref()),
            _ => None,
        };
        let element_constraint =
            expected_list_element.filter(|element| !matches!(element, Type::Any));
        let mut element_type: Option<Type> = None;

        for element in elements {
            let actual = self.check_expression(element, element_constraint);
            if actual.is_error() {
                continue;
            }
            if matches!(actual, Type::Void) {
                self.error("Void values cannot be list elements", element.span);
                continue;
            }
            if let Some(first) = &element_type {
                if first != &actual && element_constraint.is_none() {
                    self.error(
                        format!("list elements must have one type, found {first} and {actual}"),
                        element.span,
                    );
                }
            } else {
                element_type = Some(actual);
            }
        }

        if let Some(expected_element) = expected_list_element {
            if elements.is_empty() || matches!(expected_element, Type::Any) {
                return Type::List(Box::new(
                    element_type.unwrap_or_else(|| expected_element.clone()),
                ));
            }
            // The declaration's expected type is retained even when an element
            // mismatch was already diagnosed above, avoiding a duplicate error.
            return Type::List(Box::new(expected_element.clone()));
        }

        match element_type {
            Some(element) => Type::List(Box::new(element)),
            None if elements.is_empty() => {
                self.error("cannot infer the element type of an empty list", span);
                Type::Error
            }
            None => Type::List(Box::new(Type::Error)),
        }
    }

    fn check_construction(
        &mut self,
        type_name: &Identifier,
        fields: &[crate::ast::FieldInitializer],
    ) -> Type {
        let Some(info) = self.types.get(&type_name.name).cloned() else {
            self.error_with(
                DiagnosticCode::UnknownType,
                format!("unknown type '{}' in construction", type_name.name),
                type_name.span,
            );
            return Type::Error;
        };

        let mut initialized = HashSet::new();
        for initializer in fields {
            let Some(expected_type) = info.fields.get(&initializer.name.name).cloned() else {
                self.error(
                    format!(
                        "type '{}' has no field '{}'",
                        type_name.name, initializer.name.name
                    ),
                    initializer.name.span,
                );
                self.check_expression(&initializer.value, None);
                continue;
            };
            if !initialized.insert(initializer.name.name.as_str()) {
                self.error(
                    format!(
                        "field '{}' is initialized more than once",
                        initializer.name.name
                    ),
                    initializer.name.span,
                );
            }
            self.check_expression(&initializer.value, Some(&expected_type));
        }

        for field_name in &info.field_order {
            if !initialized.contains(field_name.as_str()) {
                self.error(
                    format!("missing initializer for field '{field_name}'"),
                    type_name.span,
                );
            }
        }
        info.ty
    }

    fn check_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        call_span: SourceSpan,
    ) -> Type {
        if let ExpressionKind::Identifier(identifier) = &callee.kind {
            if self.lookup_variable(&identifier.name).is_none() {
                match self.globals.get(&identifier.name).cloned() {
                    Some(GlobalSymbol::Type(ty)) => {
                        let signature = self.constructor_signature(&identifier.name);
                        self.expression_types
                            .insert(callee.span, Type::Function(signature.clone()));
                        self.check_call_arguments(
                            &identifier.name,
                            &signature,
                            arguments,
                            call_span,
                        );
                        return ty;
                    }
                    Some(GlobalSymbol::Function(signature)) => {
                        self.expression_types
                            .insert(callee.span, Type::Function(signature.clone()));
                        self.check_call_arguments(
                            &identifier.name,
                            &signature,
                            arguments,
                            call_span,
                        );
                        return *signature.return_type;
                    }
                    Some(GlobalSymbol::ExternalFunction(external)) => {
                        let signature = external.princi_signature();
                        self.expression_types
                            .insert(callee.span, Type::Function(signature.clone()));
                        self.check_call_arguments(
                            &identifier.name,
                            &signature,
                            arguments,
                            call_span,
                        );
                        return *signature.return_type;
                    }
                    None => {
                        self.expression_types.insert(callee.span, Type::Error);
                        self.error_with(
                            DiagnosticCode::UnknownIdentifier,
                            format!("undefined function '{}'", identifier.name),
                            identifier.span,
                        );
                        for argument in arguments {
                            self.check_expression(argument, None);
                        }
                        return Type::Error;
                    }
                }
            }
        }

        let callee_type = self.check_expression(callee, None);
        match callee_type {
            Type::Function(signature) => {
                self.check_call_arguments("function value", &signature, arguments, call_span);
                *signature.return_type
            }
            Type::Error => {
                for argument in arguments {
                    self.check_expression(argument, None);
                }
                Type::Error
            }
            other => {
                self.error(format!("cannot call a value of type {other}"), callee.span);
                for argument in arguments {
                    self.check_expression(argument, None);
                }
                Type::Error
            }
        }
    }

    fn constructor_signature(&self, type_name: &str) -> FunctionType {
        let Some(info) = self.types.get(type_name) else {
            return FunctionType::new(Vec::new(), Type::Error);
        };
        info.initializer.clone().unwrap_or_else(|| {
            let parameters = info
                .field_order
                .iter()
                .filter_map(|name| info.fields.get(name).cloned())
                .collect();
            FunctionType::new(parameters, Type::Void)
        })
    }

    fn check_call_arguments(
        &mut self,
        name: &str,
        signature: &FunctionType,
        arguments: &[Expression],
        call_span: SourceSpan,
    ) {
        if arguments.len() != signature.parameters.len() {
            self.error_with(
                DiagnosticCode::InvalidCall,
                format!(
                    "'{name}' expects {} argument(s), found {}",
                    signature.parameters.len(),
                    arguments.len()
                ),
                call_span,
            );
        }
        for (index, argument) in arguments.iter().enumerate() {
            let expected = signature.parameters.get(index);
            self.check_expression(argument, expected);
        }
    }

    fn member_type(&mut self, object_type: &Type, member: &Identifier) -> (Type, bool) {
        if let Type::List(element) = object_type {
            return match member.name.as_str() {
                "length" => (Type::Int, true),
                "add" => (
                    Type::Function(FunctionType::new(vec![(**element).clone()], Type::Void)),
                    false,
                ),
                _ => {
                    self.error_with(
                        DiagnosticCode::InvalidMember,
                        format!("type '{object_type}' has no member '{}'", member.name),
                        member.span,
                    );
                    (Type::Error, false)
                }
            };
        }
        let type_name = match object_type {
            Type::Class(name) | Type::Struct(name) => name,
            Type::Error => return (Type::Error, false),
            other => {
                self.error_with(
                    DiagnosticCode::InvalidMember,
                    format!("type {other} has no members"),
                    member.span,
                );
                return (Type::Error, false);
            }
        };

        let Some(info) = self.types.get(type_name) else {
            return (Type::Error, false);
        };
        if let Some(ty) = info.fields.get(&member.name) {
            return (ty.clone(), true);
        }
        if let Some(signature) = info.methods.get(&member.name) {
            return (Type::Function(signature.clone()), false);
        }

        self.error_with(
            DiagnosticCode::InvalidMember,
            format!("type '{type_name}' has no member '{}'", member.name),
            member.span,
        );
        (Type::Error, false)
    }

    fn is_struct_list_element_path(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Index { object, .. } => {
                matches!(
                    self.expression_types.get(&expression.span),
                    Some(Type::Struct(_))
                ) || self.is_struct_list_element_path(object)
            }
            ExpressionKind::Member { object, .. } | ExpressionKind::Group(object) => {
                self.is_struct_list_element_path(object)
            }
            _ => false,
        }
    }

    fn unary_result(&mut self, operator: UnaryOperator, operand: &Type, span: SourceSpan) -> Type {
        if operand.is_error() {
            return Type::Error;
        }
        match (operator, operand) {
            (UnaryOperator::Positive | UnaryOperator::Negative, Type::Int | Type::Float) => {
                operand.clone()
            }
            (UnaryOperator::Not, Type::Bool) => Type::Bool,
            (operator, other) => {
                self.error(
                    format!(
                        "unary operator '{}' cannot be applied to {other}",
                        unary_text(operator)
                    ),
                    span,
                );
                Type::Error
            }
        }
    }

    fn binary_result(
        &mut self,
        operator: BinaryOperator,
        left: &Type,
        right: &Type,
        span: SourceSpan,
    ) -> Type {
        if left.is_error() || right.is_error() {
            return Type::Error;
        }

        let result = match operator {
            BinaryOperator::Add if left == right && left.is_numeric() => Some(left.clone()),
            BinaryOperator::Add if left == &Type::String && right == &Type::String => {
                Some(Type::String)
            }
            BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide
                if left == right && left.is_numeric() =>
            {
                Some(left.clone())
            }
            BinaryOperator::Remainder if left == &Type::Int && right == &Type::Int => {
                Some(Type::Int)
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual
                if left == right && is_equality_type(left) =>
            {
                Some(Type::Bool)
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
                if left == right && left.is_numeric() =>
            {
                Some(Type::Bool)
            }
            BinaryOperator::And | BinaryOperator::Or
                if left == &Type::Bool && right == &Type::Bool =>
            {
                Some(Type::Bool)
            }
            _ => None,
        };

        result.unwrap_or_else(|| {
            self.error(
                format!(
                    "operator '{}' cannot be applied to {left} and {right}",
                    binary_text(operator)
                ),
                span,
            );
            Type::Error
        })
    }

    fn require_type(&mut self, expected: &Type, actual: &Type, span: SourceSpan, context: &str) {
        if !expected.accepts(actual) {
            self.error_with(
                DiagnosticCode::TypeMismatch,
                format!("expected {expected}, found {actual} ({context})"),
                span,
            );
        }
    }

    fn declare_local(
        &mut self,
        name: &str,
        identifier: &Identifier,
        ty: Type,
        mutable: bool,
        initialized: bool,
    ) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };
        if scope.variables.contains_key(name) {
            self.error_with(
                DiagnosticCode::DuplicateDeclaration,
                format!("duplicate declaration '{name}' in this scope"),
                identifier.span,
            );
            return;
        }
        scope.variables.insert(
            name.to_owned(),
            VariableSymbol {
                ty,
                mutable,
                initialized,
            },
        );
    }

    fn mark_assignment_initialized(&mut self, target: &Expression) {
        let ExpressionKind::Identifier(identifier) = ungroup_kind(&target.kind) else {
            return;
        };
        for scope in self.scopes.iter_mut().rev() {
            if let Some(variable) = scope.variables.get_mut(&identifier.name) {
                variable.initialized = true;
                return;
            }
        }
    }

    fn lookup_variable(&self, name: &str) -> Option<&VariableSymbol> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.variables.get(name))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn error(&mut self, message: impl Into<String>, span: SourceSpan) {
        self.error_with(DiagnosticCode::Semantic, message, span);
    }

    fn error_with(&mut self, code: DiagnosticCode, message: impl Into<String>, span: SourceSpan) {
        self.diagnostics
            .push(Diagnostic::at_source(code, message, self.source, span));
    }
}

fn ungroup_kind(mut kind: &ExpressionKind) -> &ExpressionKind {
    while let ExpressionKind::Group(inner) = kind {
        kind = &inner.kind;
    }
    kind
}

fn is_reserved_type_name(name: &str) -> bool {
    matches!(
        name,
        "Int" | "Float" | "Bool" | "String" | "Void" | "List" | "Int32" | "Int64" | "Float64"
    )
}

fn is_ascii_c_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some('_' | 'A'..='Z' | 'a'..='z'))
        && characters.all(|character| matches!(character, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9'))
}

fn is_reserved_external_symbol(name: &str) -> bool {
    name == "main"
        || name.starts_with("princi_")
        || matches!(
            name,
            "printf"
                | "putchar"
                | "strcmp"
                | "strlen"
                | "malloc"
                | "free"
                | "memcpy"
                | "memset"
                | "exit"
        )
}

fn is_list_literal(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::List(_) => true,
        ExpressionKind::Group(inner) => is_list_literal(inner),
        _ => false,
    }
}

fn is_equality_type(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Float | Type::Bool | Type::String)
}

fn unary_text(operator: UnaryOperator) -> &'static str {
    match operator {
        UnaryOperator::Positive => "+",
        UnaryOperator::Negative => "-",
        UnaryOperator::Not => "!",
    }
}

fn binary_text(operator: BinaryOperator) -> &'static str {
    match operator {
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
    }
}

#[cfg(test)]
mod tests {
    use super::analyze;
    use crate::ast::{Declaration, StatementKind};
    use crate::diagnostics::DiagnosticCode;
    use crate::lexer;
    use crate::parser;
    use crate::source::SourceFile;
    use crate::types::Type;

    fn analyze_text(text: &str) -> super::SemanticResult {
        let source = SourceFile::from_text("semantic.prnc", text);
        let tokens = lexer::lex(&source).expect("semantic test source should lex");
        let parsed = parser::parse(&source, tokens);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        analyze(&source, &parsed.program)
    }

    fn function<'a>(
        result: &'a super::SemanticResult,
        name: &str,
    ) -> &'a crate::ast::FunctionDeclaration {
        result
            .typed_program
            .program
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Function(function) if function.name.name == name => Some(function),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing function {name}"))
    }

    fn variable<'a>(
        result: &'a super::SemanticResult,
        function_name: &str,
        statement_index: usize,
    ) -> &'a crate::ast::VariableDeclaration {
        let StatementKind::Variable(variable) =
            &function(result, function_name).body.statements[statement_index].kind
        else {
            panic!("expected variable declaration at statement {statement_index}");
        };
        variable
    }

    fn messages(result: &super::SemanticResult) -> Vec<String> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message().to_owned())
            .collect()
    }

    fn has_message(result: &super::SemanticResult, needle: &str) -> bool {
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message().contains(needle))
    }

    #[test]
    fn infers_local_primitives_and_checks_explicit_immutable_types() {
        let result = analyze_text(
            r#"fn main() {
    var x = 10
    var name = "Octrie"
    var count: Int = 10
    let pi: Float = 3.14159
}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 0)),
            Some(&Type::Int)
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 1)),
            Some(&Type::String)
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 2)),
            Some(&Type::Int)
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 3)),
            Some(&Type::Float)
        );
    }

    #[test]
    fn registers_generic_print_and_println_builtins_for_primitive_values() {
        let result = analyze_text(
            r#"fn main() {
    print(1)
    println(1.5)
    print(true)
    println("text")
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
    }

    #[test]
    fn resolves_forward_functions_scopes_conditions_and_ranges() {
        let result = analyze_text(
            r#"fn main() {
    let total = add(1, 2)
    if total > 2 { print(total) } else { print(0) }
    while false { var local = 1 }
    for index in 0..3 { var loop_value = index }
    var shadow = 1
    { let shadow = "nested" }
    print(shadow)
}
fn add(a: Int, b: Int) -> Int { return a + b }
fn use_later(value: Later) {}
struct Later {}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 0)),
            Some(&Type::Int)
        );
        let total_expr = match &function(&result, "main").body.statements[0].kind {
            StatementKind::Variable(variable) => variable.initializer.as_ref().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(
            result.typed_program.expression_type(total_expr),
            Some(&Type::Int)
        );
        let add = function(&result, "add");
        assert_eq!(
            result.typed_program.parameter_type(&add.parameters[0]),
            Some(&Type::Int)
        );
        assert!(matches!(
            result.typed_program.symbols.global("add"),
            Some(super::GlobalSymbol::Function(_))
        ));
    }

    #[test]
    fn checks_class_struct_fields_methods_self_and_constructors() {
        let result = analyze_text(
            r#"struct Point { x: Int; y: Int }
class User {
    name: String
    init(name: String) { self.name = name }
    fn greet() { print(self.name) }
}
class Parcel {
    contents: String
    init(contents: String) { self.contents = contents }
}
fn main() {
    let point = Point(1, 2)
    let user = User("Octrie")
    print(point.x)
    user.greet()
    var mutable_point = Point { x: 3, y: 4 }
    mutable_point.x = 5
}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 0)),
            Some(&Type::Struct("Point".to_owned()))
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 1)),
            Some(&Type::Class("User".to_owned()))
        );
        let point_symbols = result
            .typed_program
            .symbols
            .type_symbols("Point")
            .expect("Point symbols should be retained for later stages");
        assert_eq!(point_symbols.fields.get("x"), Some(&Type::Int));
        assert_eq!(point_symbols.fields.get("y"), Some(&Type::Int));
    }

    #[test]
    fn checks_struct_values_in_parameters_returns_and_mutable_bindings() {
        let result = analyze_text(
            r#"struct Point { x: Float; y: Float }
fn translate(point: Point) -> Point {
    var moved = point
    moved.x = moved.x + 1.0
    return moved
}
fn main() {
    var point = Point(3.0, 4.0)
    let copy = point
    var moved = translate(copy)
    moved.y = 5.0
    print(point.x)
}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 0)),
            Some(&Type::Struct("Point".to_owned()))
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 1)),
            Some(&Type::Struct("Point".to_owned()))
        );
        assert!(matches!(
            result.typed_program.symbols.global("translate"),
            Some(super::GlobalSymbol::Function(signature))
                if signature.parameters == [Type::Struct("Point".to_owned())]
                    && signature.return_type.as_ref() == &Type::Struct("Point".to_owned())
        ));
    }

    #[test]
    fn rejects_struct_mutation_through_immutable_bindings_methods_and_recursive_layouts() {
        let result = analyze_text(
            r#"struct Node { next: Node }
struct Point {
    x: Int
    fn move() { self.x = 1 }
    init(x: Int) { self.x = x }
}
fn main() {
    let point = Point { x: 0 }
    point.x = 1
}"#,
        );

        assert!(has_message(
            &result,
            "recursive struct fields do not have a finite value layout"
        ));
        assert!(has_message(
            &result,
            "methods on structs are not supported in v0.1"
        ));
        assert!(has_message(
            &result,
            "structs do not support init constructors in v0.1"
        ));
        assert!(has_message(
            &result,
            "cannot assign through an immutable binding"
        ));
    }

    #[test]
    fn reports_duplicate_globals_parameters_locals_and_members() {
        let result = analyze_text(
            r#"fn duplicate(a: Int, a: Int) {
    let value = 1
    var value = 2
}
fn parameter_shadow(value: Int) { let value = 3 }
fn duplicate() {}
fn Same() {}
class Same {}
class Box {
    item: Int
    item: String
    fn open() {}
    fn open() {}
    init() {}
    init() {}
}
struct Box {}"#,
        );

        assert!(has_message(&result, "duplicate declaration 'duplicate'"));
        assert!(has_message(&result, "duplicate declaration 'a'"));
        assert!(has_message(&result, "duplicate declaration 'value'"));
        assert!(has_message(&result, "duplicate member 'item'"));
        assert!(has_message(&result, "duplicate member 'open'"));
        assert!(has_message(&result, "duplicate init constructor"));
        assert!(has_message(
            &result,
            "duplicate or reserved declaration 'Box'"
        ));
        assert!(has_message(
            &result,
            "duplicate or reserved declaration 'Same'"
        ));
    }

    #[test]
    fn reports_unknown_types_identifiers_and_functions_at_source_locations() {
        let result = analyze_text(
            "fn first(value: Missing) {}\nfn main() { let item = missing; absent(1) }",
        );

        assert!(has_message(&result, "unknown type 'Missing'"));
        assert!(has_message(&result, "undefined identifier 'missing'"));
        assert!(has_message(&result, "undefined function 'absent'"));
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic.location().is_some_and(|location| {
                location.file.to_string_lossy() == "semantic.prnc"
                    && location.span.end > location.span.start
            })
        }));
    }

    #[test]
    fn checks_return_types_argument_count_and_argument_types() {
        let result = analyze_text(
            r#"fn takes(value: Int) -> Int { return "wrong" }
fn missing() -> Int { return }
fn void_result() -> Void { return 1 }
fn main() {
    takes()
    takes("wrong")
}"#,
        );

        assert!(has_message(
            &result,
            "expected Int, found String (return value)"
        ));
        assert!(has_message(
            &result,
            "return statement requires a value of type Int"
        ));
        assert!(has_message(&result, "Void function cannot return a value"));
        assert!(has_message(
            &result,
            "'takes' expects 1 argument(s), found 0"
        ));
        assert!(has_message(
            &result,
            "expected Int, found String (expression)"
        ));
    }

    #[test]
    fn checks_mutability_assignments_operators_conditions_and_list_types() {
        let result = analyze_text(
            r#"fn main() {
    let fixed: Int = 1
    fixed = 2
    var number = 1
    number = "text"
    number += true
    if 1 { print("bad") }
    while "yes" {}
    let invalid = true + false
    var values = [1, 2]
    values["zero"] = 0
    var mixed = [1, "two"]
    var empty = []
    let uninitialized: Int
    for index in 0..3 { index = 4 }
    for wrong_range in 0..3.0 {}
}
fn update_parameter(value: Int) { value = 2 }"#,
        );

        assert!(has_message(
            &result,
            "cannot assign through an immutable binding"
        ));
        assert!(has_message(&result, "expected Int, found String"));
        assert!(
            has_message(&result, "operator '+='")
                || has_message(&result, "operator '+' cannot be applied to Int and Bool")
        );
        assert!(has_message(
            &result,
            "expected Bool, found Int (if condition)"
        ));
        assert!(has_message(
            &result,
            "expected Bool, found String (while condition)"
        ));
        assert!(has_message(
            &result,
            "operator '+' cannot be applied to Bool and Bool"
        ));
        assert!(has_message(
            &result,
            "expected Int, found String (list index)"
        ));
        assert!(has_message(&result, "list elements must have one type"));
        assert!(has_message(
            &result,
            "cannot infer the element type of an empty list"
        ));
        assert!(has_message(
            &result,
            "let declaration requires an initializer"
        ));
        assert!(has_message(
            &result,
            "range bounds must have the same numeric type"
        ));
    }

    #[test]
    fn tracks_definite_initialization_across_assignments_branches_and_loops() {
        let initialized = analyze_text(
            r#"fn check(flag: Bool) {
    var assigned: Int
    assigned = 4
    print(assigned)

    var branch: Int
    if flag {
        branch = 1
    } else {
        branch = 2
    }
    print(branch)
}
fn main() {
    check(true)
}"#,
        );
        assert!(
            initialized.diagnostics.is_empty(),
            "{:?}",
            initialized.diagnostics
        );

        let uninitialized = analyze_text(
            r#"fn direct() {
    var value: Int
    print(value)
}
fn compound() {
    var value: Int
    value += 1
}
fn one_branch(flag: Bool) {
    var value: Int
    if flag { value = 1 }
    print(value)
}
fn while_body() {
    var value: Int
    while false { value = 1 }
    print(value)
}
fn for_body() {
    var value: Int
    for index in 0..1 { value = index }
    print(value)
}"#,
        );
        let errors = uninitialized
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code() == DiagnosticCode::UninitializedVariable)
            .collect::<Vec<_>>();
        assert_eq!(errors.len(), 5, "{:?}", uninitialized.diagnostics);
        assert!(errors.iter().all(|diagnostic| {
            diagnostic
                .message()
                .contains("may be read before it is initialized")
                && diagnostic.location().is_some()
        }));
    }

    #[test]
    fn validates_constructor_arguments_fields_and_member_access() {
        let result = analyze_text(
            r#"struct Point { x: Int; y: Int }
class Parcel {
    contents: String
    init(contents: String) { self.contents = contents }
}
fn main() {
    let short = Point(1)
    let wrong = Point("one", 2)
    let named = Point { x: "bad", extra: 3 }
    let bad_parcel = Parcel()
    let wrong_parcel = Parcel(1)
    print(named.missing)
    named.x = 5
}"#,
        );

        assert!(has_message(
            &result,
            "'Point' expects 2 argument(s), found 1"
        ));
        assert!(has_message(
            &result,
            "'Parcel' expects 1 argument(s), found 0"
        ));
        assert!(has_message(
            &result,
            "expected String, found Int (expression)"
        ));
        assert!(has_message(
            &result,
            "expected Int, found String (expression)"
        ));
        assert!(has_message(&result, "type 'Point' has no field 'extra'"));
        assert!(has_message(&result, "missing initializer for field 'y'"));
        assert!(has_message(&result, "type 'Point' has no member 'missing'"));
        assert!(has_message(
            &result,
            "cannot assign through an immutable binding"
        ));
    }

    #[test]
    fn generic_list_annotations_support_empty_values_and_local_inference() {
        let result = analyze_text(
            "fn empty_list() -> List<Int> { return [] }\nfn first(values: List<Int>) -> Int { return values[0] }\nfn main() { var items: List<Int> = ([]); var values: List<Int> = [1, 2]; var assigned: List<Int>; assigned = []; let inferred = [3, 4] }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        for index in [0, 1, 4] {
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", index)),
                Some(&Type::List(Box::new(Type::Int)))
            );
        }
    }

    #[test]
    fn checks_list_element_types_operations_and_reference_mutability() {
        let result = analyze_text(
            r#"fn first(values: List<Int>) -> Int { return values[0] }
fn identity(values: List<Int>) -> List<Int> { return values }
fn main() {
    let values: List<Int> = [1, 2]
    values.add(3)
    values[0] = 4
    values[1] += 5
    let size = values.length
    let first_value = first(values)
    let copied_handle = identity(values)
    let inferred = [1, 2, 3]
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 0)),
            Some(&Type::List(Box::new(Type::Int)))
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 4)),
            Some(&Type::Int)
        );
    }

    #[test]
    fn rejects_bare_generic_and_mismatched_list_types() {
        let result = analyze_text(
            r#"fn main() {
    let bare: List = []
    let wrong_arity: List<Int, Float> = []
    let wrong_element: List<String> = [1]
    let mixed = [1, "hello", true]
    let uninferred = []
    var values: List<Int> = []
    values.add("wrong")
    values[0] = false
    values[1.0] = 2
    values.length = 2
}"#,
        );
        assert!(has_message(
            &result,
            "List requires exactly one type argument"
        ));
        assert!(has_message(
            &result,
            "expected String, found Int (expression)"
        ));
        assert!(has_message(&result, "list elements must have one type"));
        assert!(has_message(
            &result,
            "cannot infer the element type of an empty list"
        ));
        assert!(has_message(
            &result,
            "expected Int, found String (expression)"
        ));
        assert!(has_message(&result, "expected Int, found Bool"));
        assert!(has_message(
            &result,
            "expected Int, found Float (list index)"
        ));
        assert!(has_message(&result, "list.length is read-only"));
    }

    #[test]
    fn supports_only_list_as_a_source_generic_and_rejects_void_elements() {
        let result = analyze_text(
            "struct Box {}\nfn takes(values: Box<Int>) {}\nfn empty() -> List<Void> { return [] }\nfn main() {}",
        );
        assert!(has_message(
            &result,
            "generic type arguments are only supported for List<T> in v0.1"
        ));
        assert!(has_message(
            &result,
            "Void is only valid as a function return type"
        ));
    }

    #[test]
    fn list_index_field_mutation_of_struct_elements_requires_writeback() {
        let result = analyze_text(
            "struct Point { x: Int }\nfn main() { var points: List<Point> = [Point(1)]; points[0].x = 2 }",
        );
        assert!(has_message(
            &result,
            "assign the updated struct back to the list"
        ));
    }

    #[test]
    fn procedural_ranges_require_int_and_numeric_types_do_not_convert_implicitly() {
        let result = analyze_text(
            "fn take_float(value: Float) {}\nfn main() {\n for index in 0.0..2.0 {}\n take_float(1)\n let value: Float = 1\n}",
        );

        assert!(has_message(
            &result,
            "for loop range requires Int bounds, found Float"
        ));
        assert!(has_message(
            &result,
            "expected Float, found Int (expression)"
        ));
    }

    #[test]
    fn resolves_limited_c_abi_signatures_to_princi_call_types() {
        let result = analyze_text(
            r#"extern "C" {
    fn abs(value: Int32) -> Int32
    fn native_count() -> Int64
    fn native_ratio(value: Float64) -> Float64
    fn native_reset()
}
fn main() {
    print(abs(-17))
    print(native_count())
    print(native_ratio(2.5))
    native_reset()
}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        assert!(matches!(
            result.typed_program.symbols.global("abs"),
            Some(super::GlobalSymbol::ExternalFunction(signature))
                if signature.parameters == [crate::ffi::CAbiType::Int32]
                    && signature.return_type == crate::ffi::CAbiType::Int32
                    && signature.princi_signature().parameters == [Type::Int]
                    && signature.princi_signature().return_type.as_ref() == &Type::Int
        ));
        assert!(matches!(
            result.typed_program.symbols.global("native_ratio"),
            Some(super::GlobalSymbol::ExternalFunction(signature))
                if signature.parameters == [crate::ffi::CAbiType::Float64]
                    && signature.princi_signature().parameters == [Type::Float]
        ));
    }

    #[test]
    fn rejects_managed_and_nonprimitive_ffi_types_and_non_c_abis() {
        let result = analyze_text(
            r#"class User {}
extern "system" { fn wrong_abi(value: Int32) -> Int32 }
extern "C" {
    fn string_input(value: String)
    fn list_input(value: List<Int>)
    fn object_input(value: User)
    fn boolean_output() -> Bool
    fn void_input(value: Void)
}
fn ordinary(value: Int32) {}
fn main() {}"#,
        );

        assert!(has_message(
            &result,
            "only extern \"C\" is supported in v0.1"
        ));
        assert!(has_message(&result, "type 'String' is not FFI-safe"));
        assert!(has_message(&result, "type 'List' is not FFI-safe"));
        assert!(has_message(&result, "type 'User' is not FFI-safe"));
        assert!(has_message(&result, "type 'Bool' is not FFI-safe"));
        assert!(has_message(
            &result,
            "Void is only valid as an extern function return type"
        ));
        assert!(has_message(
            &result,
            "C ABI types may only be used in extern \"C\" declarations"
        ));
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic
                .location()
                .is_some_and(|location| location.span.end > location.span.start)
        }));
    }

    #[test]
    fn external_c_functions_cannot_escape_as_function_values() {
        let result = analyze_text(
            r#"extern "C" { fn abs(value: Int32) -> Int32 }
fn main() {
    let alias = abs
    alias(-1)
}"#,
        );

        assert!(has_message(
            &result,
            "external C functions must be called directly in v0.1"
        ));
    }
}
