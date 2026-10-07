use std::collections::{HashMap, HashSet};

use crate::ast::{
    Assignment, AssignmentOperator, BinaryOperator, Block, ClassMember, Declaration, Expression,
    ExpressionKind, ExternBlockDeclaration, FieldDeclaration, FunctionDeclaration, Identifier,
    InitializerDeclaration, Literal, Program, Statement, StatementKind, TypeDeclaration,
    TypeDeclarationKind, TypeParameterDeclaration, UnaryOperator, VariableDeclaration,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::ffi::{CAbiType, ExternalFunctionType};
use crate::modules::ResolvedModules;
use crate::source::{SourceFile, SourceSpan};
use crate::types::{FunctionType, GenericTypeConstructor, GenericTypeParameter, Type};

#[derive(Debug, Clone)]
pub struct TypedProgram {
    pub program: Program,
    pub symbols: SymbolTable,
    pub modules: ResolvedModules,
    expression_types: HashMap<SourceSpan, Type>,
    variable_types: HashMap<SourceSpan, Type>,
    generic_function_calls: HashMap<SourceSpan, GenericFunctionCall>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericFunctionCall {
    pub name: String,
    pub type_arguments: Vec<Type>,
    pub signature: FunctionType,
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

    pub fn generic_function_call(&self, span: SourceSpan) -> Option<&GenericFunctionCall> {
        self.generic_function_calls.get(&span)
    }

    pub(crate) fn specialized_view(&self, substitutions: &HashMap<u32, Type>) -> Self {
        let mut specialized = self.clone();
        for ty in specialized.expression_types.values_mut() {
            *ty = substitute_type(ty, substitutions);
        }
        for ty in specialized.variable_types.values_mut() {
            *ty = substitute_type(ty, substitutions);
        }
        for call in specialized.generic_function_calls.values_mut() {
            call.type_arguments = call
                .type_arguments
                .iter()
                .map(|ty| substitute_type(ty, substitutions))
                .collect();
            call.signature = substitute_function_type(&call.signature, substitutions);
        }
        specialized
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
    pub type_parameters: Vec<GenericTypeParameter>,
    pub fields: HashMap<String, Type>,
    field_spans: HashMap<String, SourceSpan>,
    pub field_order: Vec<String>,
    pub methods: HashMap<String, FunctionType>,
    pub initializer: Option<FunctionType>,
    member_names: HashSet<String>,
}

impl TypeSymbols {
    fn new(ty: Type, type_parameters: Vec<GenericTypeParameter>) -> Self {
        Self {
            ty,
            type_parameters,
            fields: HashMap::new(),
            field_spans: HashMap::new(),
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
    type_parameter_scopes: Vec<HashMap<String, GenericTypeParameter>>,
    declared_type_parameter_names: HashSet<String>,
    next_type_parameter_id: u32,
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
    generic_function_calls: HashMap<SourceSpan, GenericFunctionCall>,
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
            type_parameter_scopes: Vec::new(),
            declared_type_parameter_names: HashSet::new(),
            next_type_parameter_id: 0,
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
            generic_function_calls: HashMap::new(),
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
                generic_function_calls: self.generic_function_calls,
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
                    let type_parameters =
                        self.declare_type_parameters(&type_declaration.type_parameters);
                    self.globals
                        .insert(name.clone(), GlobalSymbol::Type(ty.clone()));
                    self.types
                        .insert(name.clone(), TypeSymbols::new(ty, type_parameters));
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

            let signature = self.function_signature(
                &function.type_parameters,
                &function.parameters,
                function.return_type.as_ref(),
            );
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

            let type_parameters = self
                .types
                .get(&type_declaration.name.name)
                .map(|info| info.type_parameters.clone())
                .unwrap_or_default();
            self.push_type_parameters(&type_parameters);

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
            self.pop_type_parameters();
        }
    }

    fn declare_type_parameters(
        &mut self,
        declarations: &[TypeParameterDeclaration],
    ) -> Vec<GenericTypeParameter> {
        let mut seen = HashSet::new();
        let mut parameters = Vec::new();
        for declaration in declarations {
            let name = &declaration.name.name;
            if !seen.insert(name.clone()) {
                self.error_with(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate generic parameter '{name}'"),
                    declaration.name.span,
                );
                continue;
            }
            let parameter = GenericTypeParameter {
                id: self.next_type_parameter_id,
                name: name.clone(),
            };
            self.next_type_parameter_id = self.next_type_parameter_id.saturating_add(1);
            self.declared_type_parameter_names.insert(name.clone());
            parameters.push(parameter);
        }
        parameters
    }

    fn push_type_parameters(&mut self, parameters: &[GenericTypeParameter]) {
        self.type_parameter_scopes.push(
            parameters
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.clone()))
                .collect(),
        );
    }

    fn pop_type_parameters(&mut self) {
        self.type_parameter_scopes.pop();
    }

    fn validate_struct_value_layouts(&mut self, program: &Program) {
        fn recursive_inline_span(
            ty: &Type,
            types: &HashMap<String, TypeSymbols>,
            active: &mut Vec<String>,
            incoming_span: Option<SourceSpan>,
        ) -> Option<SourceSpan> {
            if ty.list_element().is_some() || matches!(ty, Type::Class(_)) {
                return None;
            }
            let Some(name) = ty.nominal_name() else {
                return None;
            };
            let Some(info) = types.get(name) else {
                return None;
            };
            if !ty.is_struct() {
                return None;
            }
            if active.iter().any(|item| item == name) {
                return incoming_span;
            }

            active.push(name.to_owned());
            let substitutions = type_substitutions(&info.type_parameters, ty.generic_arguments());
            for field_name in &info.field_order {
                let Some(field_type) = info.fields.get(field_name) else {
                    continue;
                };
                let instantiated_field = substitute_type(field_type, &substitutions);
                let field_span = info.field_spans.get(field_name).copied();
                if let Some(span) =
                    recursive_inline_span(&instantiated_field, types, active, field_span)
                {
                    return Some(span);
                }
            }
            active.pop();
            None
        }

        let type_symbols = self.types.clone();
        for declaration in &program.declarations {
            let Declaration::Struct(structure) = declaration else {
                continue;
            };
            if !self.accepted_type_declarations.contains(&structure.span) {
                continue;
            }
            let Some(info) = type_symbols.get(&structure.name.name) else {
                continue;
            };
            let root_type = if info.type_parameters.is_empty() {
                info.ty.clone()
            } else {
                Type::generic_instance(
                    GenericTypeConstructor::Struct(structure.name.name.clone()),
                    info.type_parameters
                        .iter()
                        .cloned()
                        .map(Type::TypeParameter)
                        .collect(),
                )
            };
            if let Some(span) =
                recursive_inline_span(&root_type, &type_symbols, &mut Vec::new(), None)
            {
                self.error(
                    "recursive struct fields do not have a finite value layout",
                    span,
                );
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
            info.field_spans.insert(field.name.name.clone(), field.span);
        }
    }

    fn register_method(&mut self, owner: &TypeDeclaration, method: &FunctionDeclaration) {
        if !self.claim_member_name(owner, &method.name) {
            return;
        }
        let signature = self.function_signature(
            &method.type_parameters,
            &method.parameters,
            method.return_type.as_ref(),
        );
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

        let signature = self.function_signature(&[], &initializer.parameters, None);
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
        type_parameter_declarations: &[TypeParameterDeclaration],
        parameters: &[crate::ast::Parameter],
        return_type: Option<&crate::ast::TypeReference>,
    ) -> FunctionType {
        let type_parameters = self.declare_type_parameters(type_parameter_declarations);
        self.push_type_parameters(&type_parameters);
        let parameters = parameters
            .iter()
            .map(|parameter| self.resolve_type_reference(&parameter.type_reference, false))
            .collect();
        let return_type = return_type
            .map(|type_reference| self.resolve_type_reference(type_reference, true))
            .unwrap_or(Type::Void);
        self.pop_type_parameters();
        FunctionType::generic(type_parameters, parameters, return_type)
    }

    fn resolve_type_reference(
        &mut self,
        type_reference: &crate::ast::TypeReference,
        allow_void: bool,
    ) -> Type {
        let active_parameter = self.lookup_type_parameter(&type_reference.name);
        let arguments = type_reference
            .arguments
            .iter()
            .map(|argument| self.resolve_type_reference(argument, false))
            .collect::<Vec<_>>();

        let ty = if let Some(parameter) = active_parameter {
            if !arguments.is_empty() {
                self.error(
                    format!(
                        "type parameter '{}' cannot take generic arguments",
                        parameter.name
                    ),
                    type_reference.span,
                );
                Type::Error
            } else {
                Type::TypeParameter(parameter)
            }
        } else {
            self.resolve_named_type(type_reference, &arguments)
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

    fn resolve_named_type(&mut self, reference: &crate::ast::TypeReference, args: &[Type]) -> Type {
        let expected_arity = match reference.name.as_str() {
            "Int" | "Float" | "Bool" | "String" | "Void" | "Int32" | "Int64" | "Float64" => Some(0),
            "List" => Some(1),
            name => self.types.get(name).map(|info| info.type_parameters.len()),
        };

        if let Some(expected) = expected_arity {
            if args.len() != expected {
                self.error(
                    format!(
                        "type '{}' expects {expected} generic argument(s), found {}",
                        reference.name,
                        args.len()
                    ),
                    reference.span,
                );
                return Type::Error;
            }
        }

        if args.iter().any(Type::is_error) {
            return Type::Error;
        }

        match reference.name.as_str() {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "String" => Type::String,
            "Void" => Type::Void,
            "Int32" | "Int64" | "Float64" => {
                self.error(
                    "C ABI types may only be used in extern \"C\" declarations",
                    reference.span,
                );
                Type::Error
            }
            "List" => {
                if args.first().is_some_and(|ty| ty == &Type::Void) {
                    self.error("List elements cannot have type Void", reference.span);
                    Type::Error
                } else {
                    Type::list(args[0].clone())
                }
            }
            name => {
                let Some(info) = self.types.get(name) else {
                    let message = if self.declared_type_parameter_names.contains(name) {
                        format!("type parameter '{name}' is outside its declaration scope")
                    } else {
                        format!("unknown type '{name}'")
                    };
                    self.error_with(DiagnosticCode::UnknownType, message, reference.span);
                    return Type::Error;
                };
                if info.type_parameters.is_empty() {
                    info.ty.clone()
                } else {
                    let constructor = match &info.ty {
                        Type::Class(_) => GenericTypeConstructor::Class(name.to_owned()),
                        Type::Struct(_) => GenericTypeConstructor::Struct(name.to_owned()),
                        _ => return Type::Error,
                    };
                    Type::generic_instance(constructor, args.to_vec())
                }
            }
        }
    }

    fn lookup_type_parameter(&self, name: &str) -> Option<GenericTypeParameter> {
        self.type_parameter_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
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
        let Some(info) = self.types.get(&type_declaration.name.name) else {
            return;
        };
        let owner_type = if info.type_parameters.is_empty() {
            info.ty.clone()
        } else {
            let constructor = match &info.ty {
                Type::Class(name) => GenericTypeConstructor::Class(name.clone()),
                Type::Struct(name) => GenericTypeConstructor::Struct(name.clone()),
                _ => return,
            };
            Type::generic_instance(
                constructor,
                info.type_parameters
                    .iter()
                    .cloned()
                    .map(Type::TypeParameter)
                    .collect(),
            )
        };
        let type_parameters = info.type_parameters.clone();
        self.push_type_parameters(&type_parameters);

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
        self.pop_type_parameters();
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
        self.push_type_parameters(&signature.type_parameters);
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
        self.pop_type_parameters();
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
                let expected = (target_type.list_element().is_some()
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
                if object_type.list_element().is_some() && member.name == "length" {
                    self.error("list.length is read-only", member.span);
                    return (Type::Error, false);
                }
                if object_type.is_struct() && self.is_struct_list_element_path(object) {
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
                if let Some(element) = object_type.list_element() {
                    // Lists are heap-backed reference values: their elements
                    // remain mutable through a `let` binding.
                    (element.clone(), true)
                } else {
                    match object_type {
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
                    Some(ty) if ty.list_element().is_some()
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
                let expected = (return_type.list_element().is_some()
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
            ExpressionKind::GenericReference { name, arguments } => {
                self.check_generic_reference(name, arguments)
            }
            ExpressionKind::Construction {
                type_reference,
                fields,
            } => self.check_construction(type_reference, fields, expression.span),
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
                if let Some(element) = object_type.clone().into_list_element() {
                    element
                } else {
                    match object_type {
                        Type::Error => Type::Error,
                        other => {
                            self.error(
                                format!("cannot index a value of type {other}"),
                                object.span,
                            );
                            Type::Error
                        }
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
        let expected_list_element = expected.and_then(Type::list_element);
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
                return Type::list(element_type.unwrap_or_else(|| expected_element.clone()));
            }
            // The declaration's expected type is retained even when an element
            // mismatch was already diagnosed above, avoiding a duplicate error.
            return Type::list(expected_element.clone());
        }

        match element_type {
            Some(element) => Type::list(element),
            None if elements.is_empty() => {
                self.error("cannot infer the element type of an empty list", span);
                Type::Error
            }
            None => Type::list(Type::Error),
        }
    }

    fn check_construction(
        &mut self,
        type_reference: &crate::ast::TypeReference,
        fields: &[crate::ast::FieldInitializer],
        _construction_span: SourceSpan,
    ) -> Type {
        let instantiated_type = self.resolve_type_reference(type_reference, false);
        let Some(type_name) = instantiated_type.nominal_name().map(str::to_owned) else {
            if !instantiated_type.is_error() {
                self.error(
                    format!("type '{}' cannot be constructed", type_reference.name),
                    type_reference.span,
                );
            }
            for initializer in fields {
                self.check_expression(&initializer.value, None);
            }
            return Type::Error;
        };
        let Some(info) = self.types.get(&type_name).cloned() else {
            self.error_with(
                DiagnosticCode::UnknownType,
                format!("unknown type '{}' in construction", type_name),
                type_reference.span,
            );
            return Type::Error;
        };
        let substitutions =
            type_substitutions(&info.type_parameters, instantiated_type.generic_arguments());

        let mut initialized = HashSet::new();
        for initializer in fields {
            let Some(expected_type) = info.fields.get(&initializer.name.name).cloned() else {
                self.error(
                    format!(
                        "type '{}' has no field '{}'",
                        type_name, initializer.name.name
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
            let expected_type = substitute_type(&expected_type, &substitutions);
            self.check_expression(&initializer.value, Some(&expected_type));
        }

        for field_name in &info.field_order {
            if !initialized.contains(field_name.as_str()) {
                self.error(
                    format!("missing initializer for field '{field_name}'"),
                    type_reference.span,
                );
            }
        }
        instantiated_type
    }

    fn check_generic_reference(
        &mut self,
        name: &Identifier,
        arguments: &[crate::ast::TypeReference],
    ) -> Type {
        for argument in arguments {
            self.resolve_type_reference(argument, false);
        }
        self.error(
            "generic type arguments are only valid on a generic function or type constructor",
            name.span,
        );
        Type::Error
    }

    fn check_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        call_span: SourceSpan,
    ) -> Type {
        if let ExpressionKind::GenericReference {
            name,
            arguments: type_arguments,
        } = &callee.kind
        {
            let resolved_arguments = type_arguments
                .iter()
                .map(|argument| self.resolve_type_reference(argument, false))
                .collect::<Vec<_>>();
            return if self.lookup_variable(&name.name).is_none() {
                match self.globals.get(&name.name).cloned() {
                    Some(GlobalSymbol::Type(_)) => {
                        let Some(info) = self.types.get(&name.name).cloned() else {
                            return Type::Error;
                        };
                        let ty = self.instantiate_declared_type(
                            &name.name,
                            &resolved_arguments,
                            name.span,
                        );
                        let signature = self.constructor_signature(&name.name);
                        let substitutions =
                            type_substitutions(&info.type_parameters, &resolved_arguments);
                        let signature = substitute_function_type(&signature, &substitutions);
                        self.expression_types
                            .insert(callee.span, Type::Function(signature.clone()));
                        self.check_call_arguments(&name.name, &signature, arguments, call_span);
                        if ty.is_error() || info.ty.is_error() {
                            Type::Error
                        } else {
                            ty
                        }
                    }
                    Some(GlobalSymbol::Function(signature)) => {
                        let (signature, type_arguments) = self.instantiate_signature(
                            &name.name,
                            &signature,
                            &resolved_arguments,
                            arguments,
                            call_span,
                        );
                        if let Some(type_arguments) = type_arguments {
                            self.generic_function_calls.insert(
                                call_span,
                                GenericFunctionCall {
                                    name: name.name.clone(),
                                    type_arguments,
                                    signature: signature.clone(),
                                },
                            );
                        }
                        self.expression_types
                            .insert(callee.span, Type::Function(signature.clone()));
                        self.check_call_arguments(&name.name, &signature, arguments, call_span);
                        *signature.return_type
                    }
                    Some(GlobalSymbol::ExternalFunction(_)) => {
                        self.error(
                            "C external functions do not accept generic type arguments",
                            name.span,
                        );
                        for argument in arguments {
                            self.check_expression(argument, None);
                        }
                        Type::Error
                    }
                    None => {
                        self.error_with(
                            DiagnosticCode::UnknownIdentifier,
                            format!("undefined function or type '{}'", name.name),
                            name.span,
                        );
                        for argument in arguments {
                            self.check_expression(argument, None);
                        }
                        Type::Error
                    }
                }
            } else {
                self.error(
                    "generic type arguments cannot be applied to a local value",
                    name.span,
                );
                for argument in arguments {
                    self.check_expression(argument, None);
                }
                Type::Error
            };
        } else if let ExpressionKind::Identifier(identifier) = &callee.kind {
            if self.lookup_variable(&identifier.name).is_none() {
                match self.globals.get(&identifier.name).cloned() {
                    Some(GlobalSymbol::Type(ty)) => {
                        let Some(info) = self.types.get(&identifier.name) else {
                            return Type::Error;
                        };
                        if !info.type_parameters.is_empty() {
                            self.error(
                                format!(
                                    "type '{}' requires {} generic argument(s) for construction",
                                    identifier.name,
                                    info.type_parameters.len()
                                ),
                                identifier.span,
                            );
                            for argument in arguments {
                                self.check_expression(argument, None);
                            }
                            return Type::Error;
                        }
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
                        let (signature, type_arguments) = self.instantiate_signature(
                            &identifier.name,
                            &signature,
                            &[],
                            arguments,
                            call_span,
                        );
                        if let Some(type_arguments) = type_arguments {
                            self.generic_function_calls.insert(
                                call_span,
                                GenericFunctionCall {
                                    name: identifier.name.clone(),
                                    type_arguments,
                                    signature: signature.clone(),
                                },
                            );
                        }
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
            let callee_type = self.check_expression(callee, None);
            return match callee_type {
                Type::Function(signature) => {
                    let (signature, _) = self.instantiate_signature(
                        "function value",
                        &signature,
                        &[],
                        arguments,
                        call_span,
                    );
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
            };
        }

        let callee_type = self.check_expression(callee, None);
        match callee_type {
            Type::Function(signature) => {
                let (signature, _) = self.instantiate_signature(
                    "function value",
                    &signature,
                    &[],
                    arguments,
                    call_span,
                );
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
        let signature = info.initializer.clone().unwrap_or_else(|| {
            let parameters = info
                .field_order
                .iter()
                .filter_map(|name| info.fields.get(name).cloned())
                .collect();
            FunctionType::new(parameters, Type::Void)
        });
        FunctionType::generic(
            info.type_parameters.clone(),
            signature.parameters,
            signature.return_type.as_ref().clone(),
        )
    }

    fn instantiate_declared_type(
        &mut self,
        name: &str,
        arguments: &[Type],
        span: SourceSpan,
    ) -> Type {
        let Some(info) = self.types.get(name).cloned() else {
            self.error_with(
                DiagnosticCode::UnknownType,
                format!("unknown type '{name}'"),
                span,
            );
            return Type::Error;
        };
        if arguments.len() != info.type_parameters.len() {
            self.error(
                format!(
                    "type '{name}' expects {} generic argument(s), found {}",
                    info.type_parameters.len(),
                    arguments.len()
                ),
                span,
            );
            return Type::Error;
        }
        if arguments.iter().any(Type::is_error) {
            return Type::Error;
        }
        if arguments.is_empty() {
            return info.ty;
        }
        let constructor = match info.ty {
            Type::Class(_) => GenericTypeConstructor::Class(name.to_owned()),
            Type::Struct(_) => GenericTypeConstructor::Struct(name.to_owned()),
            _ => return Type::Error,
        };
        Type::generic_instance(constructor, arguments.to_vec())
    }

    fn instantiate_signature(
        &mut self,
        name: &str,
        signature: &FunctionType,
        explicit_arguments: &[Type],
        call_arguments: &[Expression],
        call_span: SourceSpan,
    ) -> (FunctionType, Option<Vec<Type>>) {
        if signature.type_parameters.is_empty() {
            if !explicit_arguments.is_empty() {
                self.error(
                    format!("'{name}' does not declare generic parameters"),
                    call_span,
                );
            }
            return (signature.clone(), None);
        }

        let mut substitutions = HashMap::new();
        if !explicit_arguments.is_empty() {
            if explicit_arguments.len() != signature.type_parameters.len() {
                self.error(
                    format!(
                        "'{name}' expects {} generic argument(s), found {}",
                        signature.type_parameters.len(),
                        explicit_arguments.len()
                    ),
                    call_span,
                );
            }
            for (parameter, argument) in signature
                .type_parameters
                .iter()
                .zip(explicit_arguments.iter())
            {
                substitutions.insert(parameter.id, argument.clone());
            }
        } else {
            let diagnostic_checkpoint = self.diagnostics.len();
            let actuals = call_arguments
                .iter()
                .map(|argument| self.check_expression(argument, None))
                .collect::<Vec<_>>();
            self.diagnostics.truncate(diagnostic_checkpoint);
            for (pattern, actual) in signature.parameters.iter().zip(actuals.iter()) {
                infer_type_arguments(
                    pattern,
                    actual,
                    &signature.type_parameters,
                    &mut substitutions,
                );
            }
        }
        for parameter in &signature.type_parameters {
            if !substitutions.contains_key(&parameter.id) {
                self.error(
                    format!(
                        "cannot infer generic type argument '{}' for '{name}'",
                        parameter.name
                    ),
                    call_span,
                );
                substitutions.insert(parameter.id, Type::Error);
            }
        }
        let type_arguments = signature
            .type_parameters
            .iter()
            .map(|parameter| substitutions[&parameter.id].clone())
            .collect();
        let specialized = substitute_function_type(signature, &substitutions);
        (
            FunctionType::new(
                specialized.parameters,
                specialized.return_type.as_ref().clone(),
            ),
            Some(type_arguments),
        )
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
        if let Some(element) = object_type.list_element() {
            return match member.name.as_str() {
                "length" => (Type::Int, true),
                "add" => (
                    Type::Function(FunctionType::new(vec![element.clone()], Type::Void)),
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
        let Some(type_name) = object_type.nominal_name() else {
            if matches!(object_type, Type::Error) {
                return (Type::Error, false);
            }
            self.error_with(
                DiagnosticCode::InvalidMember,
                format!("type {object_type} has no members"),
                member.span,
            );
            return (Type::Error, false);
        };
        let Some(info) = self.types.get(type_name).cloned() else {
            return (Type::Error, false);
        };
        let substitutions = info
            .type_parameters
            .iter()
            .zip(object_type.generic_arguments())
            .map(|(parameter, argument)| (parameter.id, argument.clone()))
            .collect::<HashMap<_, _>>();
        if let Some(ty) = info.fields.get(&member.name) {
            return (substitute_type(ty, &substitutions), true);
        }
        if let Some(signature) = info.methods.get(&member.name) {
            return (
                Type::Function(substitute_function_type(signature, &substitutions)),
                false,
            );
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
                self.expression_types
                    .get(&expression.span)
                    .is_some_and(Type::is_struct)
                    || self.is_struct_list_element_path(object)
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

fn type_substitutions(
    parameters: &[GenericTypeParameter],
    arguments: &[Type],
) -> HashMap<u32, Type> {
    parameters
        .iter()
        .zip(arguments)
        .map(|(parameter, argument)| (parameter.id, argument.clone()))
        .collect()
}

fn substitute_function_type(
    signature: &FunctionType,
    substitutions: &HashMap<u32, Type>,
) -> FunctionType {
    FunctionType::generic(
        signature.type_parameters.clone(),
        signature
            .parameters
            .iter()
            .map(|ty| substitute_type(ty, substitutions))
            .collect(),
        substitute_type(&signature.return_type, substitutions),
    )
}

fn substitute_type(ty: &Type, substitutions: &HashMap<u32, Type>) -> Type {
    match ty {
        Type::TypeParameter(parameter) => substitutions
            .get(&parameter.id)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        Type::GenericInstance {
            constructor,
            arguments,
        } => Type::generic_instance(
            constructor.clone(),
            arguments
                .iter()
                .map(|argument| substitute_type(argument, substitutions))
                .collect(),
        ),
        Type::Function(signature) => {
            Type::Function(substitute_function_type(signature, substitutions))
        }
        Type::Range(element) => Type::Range(Box::new(substitute_type(element, substitutions))),
        _ => ty.clone(),
    }
}

fn infer_type_arguments(
    pattern: &Type,
    actual: &Type,
    parameters: &[GenericTypeParameter],
    substitutions: &mut HashMap<u32, Type>,
) {
    match (pattern, actual) {
        (Type::TypeParameter(parameter), actual)
            if parameters.iter().any(|item| item.id == parameter.id) =>
        {
            substitutions
                .entry(parameter.id)
                .or_insert_with(|| actual.clone());
        }
        (
            Type::GenericInstance {
                constructor: pattern_constructor,
                arguments: pattern_arguments,
            },
            Type::GenericInstance {
                constructor: actual_constructor,
                arguments: actual_arguments,
            },
        ) if pattern_constructor == actual_constructor
            && pattern_arguments.len() == actual_arguments.len() =>
        {
            for (pattern, actual) in pattern_arguments.iter().zip(actual_arguments) {
                infer_type_arguments(pattern, actual, parameters, substitutions);
            }
        }
        (Type::Error, _) | (_, Type::Error) => {}
        _ => {}
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
    use crate::types::{GenericTypeConstructor, Type};

    fn analyze_text(text: &str) -> super::SemanticResult {
        analyze_source("semantic.prnc", text)
    }

    fn analyze_source(path: &str, text: &str) -> super::SemanticResult {
        let source = SourceFile::from_text(path, text);
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
                Some(&Type::list(Type::Int))
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
            Some(&Type::list(Type::Int))
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
            "type 'List' expects 1 generic argument(s), found 0"
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
    fn rejects_wrong_generic_arity_and_void_generic_arguments() {
        let result = analyze_text(
            "struct Box {}\nfn takes(values: Box<Int>) {}\nfn empty() -> List<Void> { return [] }\nfn main() {}",
        );
        assert!(has_message(
            &result,
            "type 'Box' expects 0 generic argument(s), found 1"
        ));
        assert!(has_message(
            &result,
            "Void is only valid as a function return type"
        ));
    }

    #[test]
    fn resolves_generic_classes_structs_functions_and_nested_instances() {
        let text = r#"class Box<T> {
    value: T
    init(value: T) { self.value = value }
}
struct Pair<A, B> {
    first: A
    second: B
}
fn identity<T>(value: T) -> T { return value }
fn accept(value: Box<List<Int>>) {}
fn main() {
    let boxed: Box<Int> = Box<Int>(7)
    let nested: Box<List<Int>> = Box<List<Int>> { value: [1, 2] }
    let pair: Pair<String, Float> = Pair<String, Float> { first: "p", second: 2.5 }
    let inferred = identity(10)
    let explicit = identity<String>("hello")
    var writable: Box<Int> = Box<Int>(2)
    writable.value = 4
    let boxed_value: Int = boxed.value
    let nested_value: Int = nested.value[0]
    let pair_first: String = pair.first
}"#;

        for extension in ["prnc", "princi"] {
            let result = analyze_source(&format!("generic.{extension}"), text);
            assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
            let Some(super::GlobalSymbol::Function(signature)) =
                result.typed_program.symbols.global("identity")
            else {
                panic!("generic function signature was not registered");
            };
            assert_eq!(signature.type_parameters.len(), 1);
            assert_eq!(
                signature.parameters[0],
                Type::TypeParameter(signature.type_parameters[0].clone())
            );
            assert_eq!(
                signature.return_type.as_ref(),
                &Type::TypeParameter(signature.type_parameters[0].clone())
            );

            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 0)),
                Some(&Type::generic_instance(
                    GenericTypeConstructor::Class("Box".to_owned()),
                    vec![Type::Int]
                ))
            );
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 1)),
                Some(&Type::generic_instance(
                    GenericTypeConstructor::Class("Box".to_owned()),
                    vec![Type::list(Type::Int)]
                ))
            );
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 2)),
                Some(&Type::generic_instance(
                    GenericTypeConstructor::Struct("Pair".to_owned()),
                    vec![Type::String, Type::Float]
                ))
            );
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 3)),
                Some(&Type::Int)
            );
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 4)),
                Some(&Type::String)
            );
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 5)),
                Some(&Type::generic_instance(
                    GenericTypeConstructor::Class("Box".to_owned()),
                    vec![Type::Int]
                ))
            );
            for index in 7..=8 {
                assert_eq!(
                    result
                        .typed_program
                        .variable_type(variable(&result, "main", index)),
                    Some(&Type::Int)
                );
            }
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 9)),
                Some(&Type::String)
            );
        }
    }

    #[test]
    fn reports_duplicate_and_out_of_scope_type_parameters() {
        let result = analyze_text(
            "fn duplicate<T, T>(value: T) {}\nfn generic<T>(value: T) {}\nfn main() { let value: T = 1 }",
        );
        assert!(has_message(&result, "duplicate generic parameter 'T'"));
        assert!(has_message(
            &result,
            "type parameter 'T' is outside its declaration scope"
        ));
    }

    #[test]
    fn reports_generic_arity_unknown_arguments_and_invalid_construction() {
        let result = analyze_text(
            r#"class Box<T> { value: T }
fn identity<T>(value: T) -> T { return value }
fn use_values(value: Box<Int, String>, unknown: Box<Missing>) {}
fn main() {
    let wrong = Box<Int, String>(1)
    let bad_value = Box<Int>("not an Int")
    identity<Int, String>(1)
}"#,
        );
        assert!(has_message(
            &result,
            "type 'Box' expects 1 generic argument(s), found 2"
        ));
        assert!(has_message(&result, "unknown type 'Missing'"));
        assert!(has_message(
            &result,
            "expected Int, found String (expression)"
        ));
        assert!(has_message(
            &result,
            "'identity' expects 1 generic argument(s), found 2"
        ));
    }

    #[test]
    fn rejects_generic_calls_when_type_arguments_cannot_be_inferred() {
        let result = analyze_text(
            "fn empty<T>() -> List<T> { return [] }\nfn main() { let values = empty() }",
        );
        assert!(has_message(
            &result,
            "cannot infer generic type argument 'T' for 'empty'"
        ));

        let duplicate_error = analyze_text(
            "fn identity<T>(value: T) -> T { return value }\nfn main() { let value = identity(missing) }",
        );
        assert_eq!(
            messages(&duplicate_error)
                .iter()
                .filter(|message| message.contains("undefined identifier 'missing'"))
                .count(),
            1,
            "inference should not report an argument diagnostic twice"
        );

        let conflicting_arguments = analyze_text(
            "fn same<T>(first: T, second: T) -> T { return first }\nfn main() { let value = same(1, \"x\") }",
        );
        assert!(has_message(
            &conflicting_arguments,
            "expected Int, found String (expression)"
        ));
    }

    #[test]
    fn detects_recursive_generic_struct_layouts_but_allows_list_indirection() {
        let result = analyze_text(
            "struct Node<T> { next: Node<T> }\nstruct Tree<T> { children: List<Tree<T>> }\nfn main() {}",
        );
        assert_eq!(
            messages(&result)
                .iter()
                .filter(|message| message.contains("recursive struct fields"))
                .count(),
            1,
            "{:?}",
            messages(&result)
        );
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
