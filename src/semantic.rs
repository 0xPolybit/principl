use std::collections::{HashMap, HashSet};

use crate::ast::{
    Assignment, AssignmentOperator, BinaryOperator, Block, ClassMember, Declaration,
    EnumDeclaration, Expression, ExpressionKind, ExternBlockDeclaration, FieldDeclaration,
    FunctionDeclaration, Identifier, InitializerDeclaration, InterfaceDeclaration, Literal,
    MatchArmBody, MatchExpression, Pattern, PatternKind, Program, Statement, StatementKind,
    TraitDeclaration, TypeDeclaration, TypeDeclarationKind, TypeParameterDeclaration,
    UnaryOperator, VariableDeclaration,
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
    contract_conversions: HashMap<SourceSpan, Type>,
    result_propagations: HashMap<SourceSpan, ResultPropagation>,
    return_result_wrappings: HashMap<SourceSpan, Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericFunctionCall {
    pub name: String,
    pub type_arguments: Vec<Type>,
    pub signature: FunctionType,
}

/// Type information needed to lower postfix `?` after generic specialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultPropagation {
    pub result_type: Type,
    pub success_type: Type,
    pub error_type: Type,
    pub enclosing_result_type: Type,
}

impl TypedProgram {
    pub fn expression_type(&self, expression: &Expression) -> Option<&Type> {
        self.expression_types.get(&expression.span)
    }

    pub(crate) fn expression_type_at(&self, span: SourceSpan) -> Option<&Type> {
        self.expression_types.get(&span)
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

    pub(crate) fn contract_conversion(&self, span: SourceSpan) -> Option<&Type> {
        self.contract_conversions.get(&span)
    }

    pub(crate) fn result_propagation(&self, span: SourceSpan) -> Option<&ResultPropagation> {
        self.result_propagations.get(&span)
    }

    pub(crate) fn return_result_wrapping(&self, span: SourceSpan) -> Option<&Type> {
        self.return_result_wrappings.get(&span)
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
        for propagation in specialized.result_propagations.values_mut() {
            propagation.result_type = substitute_type(&propagation.result_type, substitutions);
            propagation.success_type = substitute_type(&propagation.success_type, substitutions);
            propagation.error_type = substitute_type(&propagation.error_type, substitutions);
            propagation.enclosing_result_type =
                substitute_type(&propagation.enclosing_result_type, substitutions);
        }
        for ty in specialized.return_result_wrappings.values_mut() {
            *ty = substitute_type(ty, substitutions);
        }
        specialized
    }

    pub(crate) fn generic_nominal_instances(&self) -> HashSet<Type> {
        let mut instances = HashSet::new();
        for ty in self
            .expression_types
            .values()
            .chain(self.variable_types.values())
        {
            collect_generic_nominal_instances(ty, &mut instances);
        }
        for symbol in self.symbols.globals.values() {
            match symbol {
                GlobalSymbol::Function(signature) => {
                    collect_function_type_instances(signature, &mut instances)
                }
                GlobalSymbol::Type(_) | GlobalSymbol::ExternalFunction(_) => {}
            }
        }
        for info in self.symbols.types.values() {
            for ty in info.fields.values() {
                collect_generic_nominal_instances(ty, &mut instances);
            }
            for signature in info.methods.values() {
                collect_function_type_instances(signature, &mut instances);
            }
            if let Some(signature) = &info.initializer {
                collect_function_type_instances(signature, &mut instances);
            }
            for variant in &info.enum_variants {
                for ty in &variant.payload {
                    collect_generic_nominal_instances(ty, &mut instances);
                }
            }
        }
        for call in self.generic_function_calls.values() {
            for ty in &call.type_arguments {
                collect_generic_nominal_instances(ty, &mut instances);
            }
            collect_function_type_instances(&call.signature, &mut instances);
        }
        instances
    }
}

fn collect_generic_nominal_instances(ty: &Type, instances: &mut HashSet<Type>) {
    match ty {
        Type::GenericInstance {
            constructor,
            arguments,
        } => {
            if matches!(
                constructor,
                GenericTypeConstructor::Class(_)
                    | GenericTypeConstructor::Struct(_)
                    | GenericTypeConstructor::Enum(_)
            ) {
                instances.insert(ty.clone());
            }
            for argument in arguments {
                collect_generic_nominal_instances(argument, instances);
            }
        }
        Type::Function(signature) => collect_function_type_instances(signature, instances),
        Type::Range(element) => collect_generic_nominal_instances(element, instances),
        _ => {}
    }
}

fn collect_function_type_instances(signature: &FunctionType, instances: &mut HashSet<Type>) {
    for parameter in &signature.parameters {
        collect_generic_nominal_instances(parameter, instances);
    }
    collect_generic_nominal_instances(&signature.return_type, instances);
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
    pub is_interface: bool,
    pub is_trait: bool,
    pub is_enum: bool,
    pub enum_variants: Vec<EnumVariantSymbols>,
    pub implemented_interfaces: Vec<String>,
    pub implemented_traits: Vec<String>,
    /// The unique default provider selected for methods not implemented by
    /// this concrete type. Ambiguous providers are rejected semantically.
    pub default_trait_methods: HashMap<String, String>,
    member_names: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct EnumVariantSymbols {
    pub name: String,
    pub payload: Vec<Type>,
    pub span: SourceSpan,
}

impl TypeSymbols {
    fn new(
        ty: Type,
        type_parameters: Vec<GenericTypeParameter>,
        is_interface: bool,
        is_trait: bool,
        is_enum: bool,
    ) -> Self {
        Self {
            ty,
            type_parameters,
            fields: HashMap::new(),
            field_spans: HashMap::new(),
            field_order: Vec::new(),
            methods: HashMap::new(),
            initializer: None,
            is_interface,
            is_trait,
            is_enum,
            enum_variants: Vec::new(),
            implemented_interfaces: Vec::new(),
            implemented_traits: Vec::new(),
            default_trait_methods: HashMap::new(),
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

#[derive(Debug, Clone)]
struct PendingGenericBoundCheck {
    parameters: Vec<GenericTypeParameter>,
    arguments: Vec<Type>,
    span: SourceSpan,
    required_by: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternCoverage {
    CatchAll,
    EnumVariant { index: usize, complete: bool },
    Literal(String),
    Partial(String),
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
    accepted_trait_methods: HashSet<SourceSpan>,
    function_signatures: HashMap<SourceSpan, FunctionType>,
    initializer_signatures: HashMap<SourceSpan, FunctionType>,
    current_return_type: Option<Type>,
    current_type: Option<Type>,
    expression_types: HashMap<SourceSpan, Type>,
    variable_types: HashMap<SourceSpan, Type>,
    generic_function_calls: HashMap<SourceSpan, GenericFunctionCall>,
    contract_conversions: HashMap<SourceSpan, Type>,
    result_propagations: HashMap<SourceSpan, ResultPropagation>,
    return_result_wrappings: HashMap<SourceSpan, Type>,
    pending_generic_bound_checks: Vec<PendingGenericBoundCheck>,
    invalid_contract_implementations: HashSet<(String, String)>,
    contracts_validated: bool,
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
            accepted_trait_methods: HashSet::new(),
            function_signatures: HashMap::new(),
            initializer_signatures: HashMap::new(),
            current_return_type: None,
            current_type: None,
            expression_types: HashMap::new(),
            variable_types: HashMap::new(),
            generic_function_calls: HashMap::new(),
            contract_conversions: HashMap::new(),
            result_propagations: HashMap::new(),
            return_result_wrappings: HashMap::new(),
            pending_generic_bound_checks: Vec::new(),
            invalid_contract_implementations: HashSet::new(),
            contracts_validated: false,
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self, program: &Program) -> SemanticResult {
        self.register_global_names(program);
        self.validate_declared_type_parameter_bounds(program);
        self.resolve_global_functions(program);
        self.register_type_members(program);
        self.validate_contract_implementations(program);
        self.contracts_validated = true;
        let pending_checks = std::mem::take(&mut self.pending_generic_bound_checks);
        for check in pending_checks {
            self.validate_generic_arguments(
                &check.parameters,
                &check.arguments,
                check.span,
                &check.required_by,
            );
        }
        self.validate_inline_value_layouts(program);
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
                contract_conversions: self.contract_conversions,
                result_propagations: self.result_propagations,
                return_result_wrappings: self.return_result_wrappings,
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
                    self.types.insert(
                        name.clone(),
                        TypeSymbols::new(ty, type_parameters, false, false, false),
                    );
                    self.accepted_type_declarations
                        .insert(type_declaration.span);
                }
                Declaration::Enum(enum_declaration) => {
                    let name = &enum_declaration.name.name;
                    if is_reserved_type_name(name) || self.globals.contains_key(name) {
                        self.error_with(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("duplicate or reserved declaration '{name}'"),
                            enum_declaration.name.span,
                        );
                        continue;
                    }
                    let ty = Type::Enum(name.clone());
                    let type_parameters =
                        self.declare_type_parameters(&enum_declaration.type_parameters);
                    self.globals
                        .insert(name.clone(), GlobalSymbol::Type(ty.clone()));
                    self.types.insert(
                        name.clone(),
                        TypeSymbols::new(ty, type_parameters, false, false, true),
                    );
                    self.accepted_type_declarations
                        .insert(enum_declaration.span);
                }
                Declaration::Interface(interface) => {
                    let name = &interface.name.name;
                    if is_reserved_type_name(name) || self.globals.contains_key(name) {
                        self.error_with(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("duplicate or reserved declaration '{name}'"),
                            interface.name.span,
                        );
                        continue;
                    }
                    let ty = Type::Interface(name.clone());
                    self.globals
                        .insert(name.clone(), GlobalSymbol::Type(ty.clone()));
                    self.types.insert(
                        name.clone(),
                        TypeSymbols::new(ty, Vec::new(), true, false, false),
                    );
                }
                Declaration::Trait(trait_declaration) => {
                    let name = &trait_declaration.name.name;
                    if is_reserved_type_name(name) || self.globals.contains_key(name) {
                        self.error_with(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("duplicate or reserved declaration '{name}'"),
                            trait_declaration.name.span,
                        );
                        continue;
                    }
                    let ty = Type::Trait(name.clone());
                    self.globals
                        .insert(name.clone(), GlobalSymbol::Type(ty.clone()));
                    self.types.insert(
                        name.clone(),
                        TypeSymbols::new(ty, Vec::new(), false, true, false),
                    );
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

    fn validate_declared_type_parameter_bounds(&mut self, program: &Program) {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(function) => {
                    self.validate_type_parameter_bounds(&function.type_parameters);
                }
                Declaration::Class(owner) | Declaration::Struct(owner) => {
                    self.validate_type_parameter_bounds(&owner.type_parameters);
                    for member in &owner.members {
                        if let ClassMember::Method(method) = member {
                            self.validate_type_parameter_bounds(&method.type_parameters);
                        }
                    }
                }
                Declaration::Enum(item) => {
                    self.validate_type_parameter_bounds(&item.type_parameters);
                }
                _ => {}
            }
        }
    }

    fn validate_type_parameter_bounds(&mut self, declarations: &[TypeParameterDeclaration]) {
        for parameter in declarations {
            let mut seen = HashSet::new();
            for bound in &parameter.bounds {
                if !seen.insert(bound.name.clone()) {
                    self.error_with(
                        DiagnosticCode::GenericConstraint,
                        format!(
                            "duplicate bound '{}' on generic parameter '{}'",
                            bound.name, parameter.name.name
                        ),
                        bound.span,
                    );
                    continue;
                }
                if !bound.arguments.is_empty() {
                    self.error_with(
                        DiagnosticCode::GenericConstraint,
                        "generic bounds cannot have type arguments in v0.2",
                        bound.span,
                    );
                    continue;
                }
                match self.types.get(&bound.name) {
                    None if matches!(
                        bound.name.as_str(),
                        "Int"
                            | "Float"
                            | "Bool"
                            | "String"
                            | "Void"
                            | "List"
                            | "Int32"
                            | "Int64"
                            | "Float64"
                    ) =>
                    {
                        self.error_with(
                            DiagnosticCode::GenericConstraint,
                            format!(
                                "generic bound '{}' must name an interface or trait",
                                bound.name
                            ),
                            bound.span,
                        );
                    }
                    None => self.error_with(
                        DiagnosticCode::UnknownType,
                        format!("unknown type '{}' in generic bound", bound.name),
                        bound.span,
                    ),
                    Some(info) if !info.is_interface && !info.is_trait => self.error_with(
                        DiagnosticCode::GenericConstraint,
                        format!(
                            "generic bound '{}' must name an interface or trait",
                            bound.name
                        ),
                        bound.span,
                    ),
                    Some(_) => {}
                }
            }
        }
    }

    fn register_external_block(&mut self, block: &ExternBlockDeclaration) {
        if block.abi != "C" {
            self.error(
                format!(
                    "unsupported extern ABI '{}'; only extern \"C\" is supported",
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
                        "type '{name}' is not FFI-safe; use Int32, Int64, Float64{}",
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
            if let Declaration::Enum(enum_declaration) = declaration {
                self.register_enum_variants(enum_declaration);
                continue;
            }
            if let Declaration::Interface(interface) = declaration {
                self.register_interface_methods(interface);
                continue;
            }
            if let Declaration::Trait(trait_declaration) = declaration {
                self.register_trait_methods(trait_declaration);
                continue;
            }
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
                        if type_declaration.kind == TypeDeclarationKind::Struct
                            && !type_declaration.interfaces.iter().any(|reference| {
                                self.types
                                    .get(&reference.name)
                                    .is_some_and(|info| info.is_trait)
                            }) =>
                    {
                        self.error_with(
                            DiagnosticCode::InvalidTrait,
                            "methods on structs are supported only when they implement a trait",
                            method.name.span,
                        )
                    }
                    ClassMember::Method(method) => self.register_method(type_declaration, method),
                    ClassMember::Initializer(initializer)
                        if type_declaration.kind == TypeDeclarationKind::Struct =>
                    {
                        self.error(
                            "structs do not support init constructors; construct them from fields",
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

    fn register_enum_variants(&mut self, declaration: &EnumDeclaration) {
        if !self.accepted_type_declarations.contains(&declaration.span) {
            return;
        }
        let parameters = self
            .types
            .get(&declaration.name.name)
            .map(|info| info.type_parameters.clone())
            .unwrap_or_default();
        self.push_type_parameters(&parameters);
        let mut variants = Vec::new();
        let mut seen = HashSet::new();
        for variant in &declaration.variants {
            if !seen.insert(variant.name.name.clone()) {
                self.error_with(
                    DiagnosticCode::DuplicateDeclaration,
                    format!(
                        "duplicate variant '{}' in enum '{}'",
                        variant.name.name, declaration.name.name
                    ),
                    variant.name.span,
                );
                continue;
            }
            let payload = variant
                .payload
                .iter()
                .map(|reference| self.resolve_type_reference(reference, false))
                .collect();
            variants.push(EnumVariantSymbols {
                name: variant.name.name.clone(),
                payload,
                span: variant.span,
            });
        }
        self.pop_type_parameters();
        if let Some(info) = self.types.get_mut(&declaration.name.name) {
            info.enum_variants = variants;
        }
    }

    fn register_interface_methods(&mut self, interface: &InterfaceDeclaration) {
        let Some(info) = self.types.get(&interface.name.name) else {
            return;
        };
        if !info.is_interface {
            return;
        }
        for method in &interface.methods {
            let name = &method.name;
            let inserted = self
                .types
                .get_mut(&interface.name.name)
                .is_some_and(|info| info.member_names.insert(name.name.clone()));
            if !inserted {
                self.error_with(
                    DiagnosticCode::DuplicateDeclaration,
                    format!(
                        "duplicate method '{}' in interface '{}'",
                        name.name, interface.name.name
                    ),
                    name.span,
                );
                continue;
            }
            let parameters = method
                .parameters
                .iter()
                .map(|parameter| self.resolve_type_reference(&parameter.type_reference, false))
                .collect::<Vec<_>>();
            let return_type = method
                .return_type
                .as_ref()
                .map(|ty| self.resolve_type_reference(ty, true))
                .unwrap_or(Type::Void);
            let signature = FunctionType::new(parameters, return_type);
            if let Some(info) = self.types.get_mut(&interface.name.name) {
                info.methods.insert(name.name.clone(), signature);
            }
        }
    }

    fn register_trait_methods(&mut self, trait_declaration: &TraitDeclaration) {
        let Some(info) = self.types.get(&trait_declaration.name.name) else {
            return;
        };
        if !info.is_trait {
            return;
        }
        for method in &trait_declaration.methods {
            if !self
                .types
                .get_mut(&trait_declaration.name.name)
                .is_some_and(|info| info.member_names.insert(method.name.name.clone()))
            {
                self.error_with(
                    DiagnosticCode::DuplicateDeclaration,
                    format!(
                        "duplicate method '{}' in trait '{}'",
                        method.name.name, trait_declaration.name.name
                    ),
                    method.name.span,
                );
                continue;
            }
            self.accepted_trait_methods.insert(method.span);
            let parameters = method
                .parameters
                .iter()
                .map(|parameter| self.resolve_type_reference(&parameter.type_reference, false))
                .collect::<Vec<_>>();
            let return_type = method
                .return_type
                .as_ref()
                .map(|ty| self.resolve_type_reference(ty, true))
                .unwrap_or(Type::Void);
            let signature = FunctionType::new(parameters, return_type);
            if let Some(info) = self.types.get_mut(&trait_declaration.name.name) {
                info.methods.insert(method.name.name.clone(), signature);
            }
        }
    }

    fn validate_contract_implementations(&mut self, program: &Program) {
        for declaration in &program.declarations {
            let (Declaration::Class(owner) | Declaration::Struct(owner)) = declaration else {
                continue;
            };
            if owner.interfaces.is_empty() {
                continue;
            }
            let parameters = self
                .types
                .get(&owner.name.name)
                .map(|info| info.type_parameters.clone())
                .unwrap_or_default();
            self.push_type_parameters(&parameters);
            let Some(owner_info) = self.types.get(&owner.name.name).cloned() else {
                self.pop_type_parameters();
                continue;
            };
            let mut seen = HashSet::new();
            let mut interfaces = Vec::new();
            let mut traits = Vec::new();
            for reference in &owner.interfaces {
                if !seen.insert(reference.name.clone()) {
                    self.error_with(
                        DiagnosticCode::DuplicateDeclaration,
                        format!(
                            "'{}' is listed more than once in implements",
                            reference.name
                        ),
                        reference.span,
                    );
                    continue;
                }
                let Some(contract) = self.types.get(&reference.name).cloned() else {
                    self.error_with(
                        DiagnosticCode::UnknownType,
                        format!("unknown interface '{}' or trait", reference.name),
                        reference.span,
                    );
                    continue;
                };
                if contract.is_interface {
                    if owner.kind == TypeDeclarationKind::Struct {
                        self.error_with(
                            DiagnosticCode::InvalidInterface,
                            "only classes can implement interfaces in v0.2",
                            reference.span,
                        );
                        continue;
                    }
                    self.resolve_type_reference(reference, false);
                    interfaces.push((reference, contract));
                } else if contract.is_trait {
                    self.resolve_type_reference(reference, false);
                    traits.push((reference, contract));
                } else {
                    self.error_with(
                        DiagnosticCode::InvalidTrait,
                        format!("'{}' is not an interface or trait", reference.name),
                        reference.span,
                    );
                }
            }
            self.pop_type_parameters();

            let mut implemented_interfaces = Vec::new();
            for (reference, interface_info) in interfaces {
                let mut conforms = true;
                let interface =
                    program
                        .declarations
                        .iter()
                        .find_map(|declaration| match declaration {
                            Declaration::Interface(candidate)
                                if candidate.name.name == reference.name =>
                            {
                                Some(candidate)
                            }
                            _ => None,
                        });
                if let Some(interface) = interface {
                    for required in &interface.methods {
                        let name = &required.name.name;
                        let required_signature = interface_info.methods.get(name);
                        match (required_signature, owner_info.methods.get(name)) {
                            (Some(expected), Some(actual)) if expected == actual => {}
                            (Some(expected), Some(actual)) => {
                                self.error_with_note(
                                    DiagnosticCode::InvalidInterface,
                                    format!(
                                        "method '{}.{}' does not match interface '{}': expected {}, found {}",
                                        owner.name.name,
                                        name,
                                        reference.name,
                                        function_signature(expected),
                                        function_signature(actual)
                                    ),
                                    self.method_span(owner, name).unwrap_or(reference.span),
                                    format!("required by interface '{}'", reference.name),
                                );
                                conforms = false;
                            }
                            (Some(expected), None) => {
                                self.error_with_note(
                                    DiagnosticCode::InvalidInterface,
                                    format!(
                                        "type '{}' is missing method '{}' required by interface '{}'",
                                        owner.name.name, name, reference.name
                                    ),
                                    reference.span,
                                    format!("required signature: {}", function_signature(expected)),
                                );
                                conforms = false;
                            }
                            (None, _) => {}
                        }
                    }
                }
                if conforms {
                    implemented_interfaces.push(reference.name.clone());
                } else {
                    self.invalid_contract_implementations
                        .insert((owner.name.name.clone(), reference.name.clone()));
                }
            }

            let mut requirements: HashMap<String, Vec<(String, FunctionType, bool, SourceSpan)>> =
                HashMap::new();
            for (reference, trait_info) in &traits {
                let Some(trait_declaration) =
                    program
                        .declarations
                        .iter()
                        .find_map(|declaration| match declaration {
                            Declaration::Trait(candidate)
                                if candidate.name.name == reference.name =>
                            {
                                Some(candidate)
                            }
                            _ => None,
                        })
                else {
                    continue;
                };
                for method in &trait_declaration.methods {
                    if let Some(signature) = trait_info.methods.get(&method.name.name) {
                        requirements
                            .entry(method.name.name.clone())
                            .or_default()
                            .push((
                                reference.name.clone(),
                                signature.clone(),
                                method.body.is_some(),
                                method.name.span,
                            ));
                    }
                }
            }

            let mut default_trait_methods = HashMap::new();
            for (method_name, methods) in &requirements {
                let explicit = owner_info.methods.get(method_name);
                let defaults = methods
                    .iter()
                    .filter(|(_, _, has_body, _)| *has_body)
                    .collect::<Vec<_>>();
                let selected_signature = if let Some(explicit) = explicit {
                    for (trait_name, expected, _, _) in methods {
                        if explicit != expected {
                            self.error_with_note(
                                DiagnosticCode::InvalidTrait,
                                format!(
                                    "method '{}.{}' does not match trait '{}': expected {}, found {}",
                                    owner.name.name,
                                    method_name,
                                    trait_name,
                                    function_signature(expected),
                                    function_signature(explicit)
                                ),
                                self.method_span(owner, method_name)
                                    .unwrap_or(owner.name.span),
                                format!("required by trait '{}'", trait_name),
                            );
                        }
                    }
                    Some(explicit.clone())
                } else if defaults.len() > 1 {
                    let providers = defaults
                        .iter()
                        .map(|(name, _, _, _)| name.as_str())
                        .collect::<Vec<_>>()
                        .join("', '");
                    self.error_with_note(
                        DiagnosticCode::InvalidTrait,
                        format!(
                            "conflicting default method '{}': traits '{}' provide it",
                            method_name, providers
                        ),
                        defaults[1].3,
                        format!(
                            "implement '{}' on '{}' to resolve the conflict",
                            method_name, owner.name.name
                        ),
                    );
                    None
                } else if let Some((trait_name, signature, _, _)) = defaults.first() {
                    for (required_trait, expected, _, _) in methods {
                        if signature != expected {
                            self.error_with_note(
                                DiagnosticCode::InvalidTrait,
                                format!(
                                    "default method '{}.{}' does not match trait '{}': expected {}, found {}",
                                    trait_name,
                                    method_name,
                                    required_trait,
                                    function_signature(expected),
                                    function_signature(signature)
                                ),
                                self.method_span(owner, method_name).unwrap_or(owner.name.span),
                                format!("required by trait '{}'", required_trait),
                            );
                        }
                    }
                    default_trait_methods.insert(method_name.clone(), trait_name.clone());
                    Some(signature.clone())
                } else {
                    None
                };

                if selected_signature.is_none() && defaults.is_empty() && explicit.is_none() {
                    for (trait_name, required, _, span) in methods {
                        self.error_with_note(
                            DiagnosticCode::InvalidTrait,
                            format!(
                                "type '{}' is missing method '{}' required by trait '{}'",
                                owner.name.name, method_name, trait_name
                            ),
                            *span,
                            format!("required signature: {}", function_signature(required)),
                        );
                    }
                }
            }

            let mut implemented_traits = Vec::new();
            for (reference, trait_info) in traits {
                let mut conforms = true;
                let trait_declaration =
                    program
                        .declarations
                        .iter()
                        .find_map(|declaration| match declaration {
                            Declaration::Trait(candidate)
                                if candidate.name.name == reference.name =>
                            {
                                Some(candidate)
                            }
                            _ => None,
                        });
                if let Some(trait_declaration) = trait_declaration {
                    for method in &trait_declaration.methods {
                        let Some(required) = trait_info.methods.get(&method.name.name) else {
                            continue;
                        };
                        let effective = owner_info.methods.get(&method.name.name).or_else(|| {
                            default_trait_methods
                                .get(&method.name.name)
                                .and_then(|provider| {
                                    self.types
                                        .get(provider)
                                        .and_then(|info| info.methods.get(&method.name.name))
                                })
                        });
                        if effective != Some(required) {
                            conforms = false;
                        }
                    }
                }
                if conforms {
                    implemented_traits.push(reference.name.clone());
                } else {
                    self.invalid_contract_implementations
                        .insert((owner.name.name.clone(), reference.name.clone()));
                }
            }
            if let Some(info) = self.types.get_mut(&owner.name.name) {
                info.implemented_interfaces = implemented_interfaces;
                info.implemented_traits = implemented_traits;
                info.default_trait_methods = default_trait_methods;
            }
        }
    }

    fn method_span(&self, owner: &TypeDeclaration, name: &str) -> Option<SourceSpan> {
        owner.members.iter().find_map(|member| match member {
            ClassMember::Method(method) if method.name.name == name => Some(method.name.span),
            _ => None,
        })
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
            let mut seen_bounds = HashSet::new();
            let parameter = GenericTypeParameter {
                id: self.next_type_parameter_id,
                name: name.clone(),
                bounds: declaration
                    .bounds
                    .iter()
                    .filter_map(|bound| {
                        seen_bounds
                            .insert(bound.name.clone())
                            .then(|| bound.name.clone())
                    })
                    .collect(),
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

    fn validate_inline_value_layouts(&mut self, program: &Program) {
        fn recursive_inline_span(
            ty: &Type,
            types: &HashMap<String, TypeSymbols>,
            active: &mut Vec<String>,
            incoming_span: Option<SourceSpan>,
        ) -> Option<SourceSpan> {
            if ty.is_managed_reference() {
                return None;
            }
            let Some(name) = ty.nominal_name() else {
                return None;
            };
            let Some(info) = types.get(name) else {
                return None;
            };
            if !ty.is_struct() && !ty.is_enum() {
                return None;
            }
            if active.iter().any(|item| item == name) {
                return incoming_span;
            }

            active.push(name.to_owned());
            let substitutions = type_substitutions(&info.type_parameters, ty.generic_arguments());
            if ty.is_struct() {
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
            } else {
                for variant in &info.enum_variants {
                    for payload_type in &variant.payload {
                        let instantiated = substitute_type(payload_type, &substitutions);
                        if let Some(span) =
                            recursive_inline_span(&instantiated, types, active, Some(variant.span))
                        {
                            return Some(span);
                        }
                    }
                }
            }
            active.pop();
            None
        }

        let type_symbols = self.types.clone();
        for declaration in &program.declarations {
            let (name, declaration_span, kind) = match declaration {
                Declaration::Struct(item) => (&item.name.name, item.span, "struct"),
                Declaration::Enum(item) => (&item.name.name, item.span, "enum"),
                _ => continue,
            };
            if !self.accepted_type_declarations.contains(&declaration_span) {
                continue;
            }
            let Some(info) = type_symbols.get(name) else {
                continue;
            };
            let root_type = if info.type_parameters.is_empty() {
                info.ty.clone()
            } else {
                let constructor = if info.is_enum {
                    GenericTypeConstructor::Enum(name.clone())
                } else {
                    GenericTypeConstructor::Struct(name.clone())
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
            if let Some(span) =
                recursive_inline_span(&root_type, &type_symbols, &mut Vec::new(), None)
            {
                let message = if kind == "struct" {
                    "recursive struct fields do not have a finite value layout"
                } else {
                    "recursive enum payloads do not have a finite value layout"
                };
                self.error(message, span);
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
                self.error_with(
                    DiagnosticCode::GenericArgument,
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
                let Some(info) = self.types.get(name).cloned() else {
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
                        Type::Enum(_) => GenericTypeConstructor::Enum(name.to_owned()),
                        _ => return Type::Error,
                    };
                    let required_by = format!(
                        "type {}",
                        generic_declaration(&reference.name, &info.type_parameters)
                    );
                    self.validate_generic_arguments(
                        &info.type_parameters,
                        args,
                        reference.span,
                        &required_by,
                    );
                    Type::generic_instance(constructor, args.to_vec())
                }
            }
        }
    }

    fn validate_generic_arguments(
        &mut self,
        parameters: &[GenericTypeParameter],
        arguments: &[Type],
        span: SourceSpan,
        required_by: &str,
    ) {
        if !self.contracts_validated {
            self.pending_generic_bound_checks
                .push(PendingGenericBoundCheck {
                    parameters: parameters.to_vec(),
                    arguments: arguments.to_vec(),
                    span,
                    required_by: required_by.to_owned(),
                });
            return;
        }
        for (parameter, argument) in parameters.iter().zip(arguments) {
            if argument.is_error() {
                continue;
            }
            let mut unsatisfied = Vec::new();
            for bound in &parameter.bounds {
                let Some(bound_info) = self.types.get(bound) else {
                    continue;
                };
                if self.type_satisfies_bound(argument, bound) {
                    continue;
                }
                if argument.nominal_name().is_some_and(|name| {
                    self.invalid_contract_implementations
                        .contains(&(name.to_owned(), bound.clone()))
                }) {
                    continue;
                }
                let kind = if bound_info.is_interface {
                    "interface"
                } else {
                    "trait"
                };
                unsatisfied.push((kind, bound.as_str()));
            }
            if unsatisfied.is_empty() {
                continue;
            }
            let message = if unsatisfied.len() == 1 {
                let (kind, bound) = unsatisfied[0];
                format!("type {argument} does not satisfy {kind} {bound}")
            } else {
                let bounds = unsatisfied
                    .iter()
                    .map(|(kind, bound)| format!("{kind} {bound}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("type {argument} does not satisfy bounds: {bounds}")
            };
            self.error_with_note(
                DiagnosticCode::GenericConstraint,
                message,
                span,
                format!("required by {required_by}"),
            );
        }
    }

    fn type_satisfies_bound(&self, argument: &Type, bound: &str) -> bool {
        match argument {
            Type::TypeParameter(parameter) => parameter.bounds.iter().any(|item| item == bound),
            Type::Interface(name) | Type::Trait(name) if name == bound => true,
            _ => {
                let Some(type_name) = argument.nominal_name() else {
                    return false;
                };
                let Some(info) = self.types.get(type_name) else {
                    return false;
                };
                if info.is_interface {
                    info.implemented_interfaces.iter().any(|item| item == bound)
                } else if info.is_trait {
                    info.implemented_traits.iter().any(|item| item == bound)
                } else if self
                    .types
                    .get(bound)
                    .is_some_and(|bound_info| bound_info.is_interface)
                {
                    info.implemented_interfaces.iter().any(|item| item == bound)
                } else {
                    info.implemented_traits.iter().any(|item| item == bound)
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
                Declaration::Trait(trait_declaration) => {
                    self.check_trait_bodies(trait_declaration);
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
                Type::Enum(name) => GenericTypeConstructor::Enum(name.clone()),
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

    fn check_trait_bodies(&mut self, trait_declaration: &TraitDeclaration) {
        let Some(info) = self.types.get(&trait_declaration.name.name).cloned() else {
            return;
        };
        for method in &trait_declaration.methods {
            if !self.accepted_trait_methods.contains(&method.span) {
                continue;
            }
            let (Some(body), Some(signature)) =
                (method.body.as_ref(), info.methods.get(&method.name.name))
            else {
                continue;
            };
            self.check_routine(
                &method.parameters,
                body,
                signature,
                Some(Type::Trait(trait_declaration.name.name.clone())),
            );
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
        self.push_type_parameters(&signature.type_parameters);
        self.push_scope();

        if let Some(owner_type) = owner {
            let mutable_self = !owner_type.is_struct() && !matches!(owner_type, Type::Trait(_));
            self.declare_local(
                "self",
                &Identifier {
                    name: "self".to_owned(),
                    span: body.span,
                },
                owner_type,
                mutable_self,
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

        if !self.record_contract_conversion(&target_type, &value_type, assignment.value.span)
            && !target_type.accepts(&value_type)
        {
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
                    "mutating fields of struct list elements is not supported; assign the updated struct back to the list",
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
            ExpressionKind::SelfValue => self
                .lookup_variable("self")
                .is_some_and(|symbol| symbol.mutable),
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
                if is_try_expression(expression) {
                    if let Some((success_type, _)) = self.result_components(&return_type) {
                        let actual = self.check_expression(expression, Some(&success_type));
                        if !actual.is_error() && success_type.accepts(&actual) {
                            self.return_result_wrappings
                                .insert(expression.span, return_type);
                        }
                    } else {
                        // The postfix expression reports that the function's
                        // return type is not a compatible Result. Avoid a
                        // second, less useful return-value mismatch here.
                        self.check_expression(expression, None);
                    }
                    return;
                }
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

    fn enum_type_from_expression(&mut self, expression: &Expression) -> Option<Type> {
        let (name, type_arguments) = match &expression.kind {
            ExpressionKind::Identifier(identifier) => (&identifier.name, None),
            ExpressionKind::GenericReference { name, arguments } => {
                (&name.name, Some(arguments.as_slice()))
            }
            _ => return None,
        };
        if self.lookup_variable(name).is_some() {
            return None;
        }
        if !self.types.get(name).is_some_and(|info| info.is_enum) {
            return None;
        }
        let arguments = type_arguments
            .unwrap_or_default()
            .iter()
            .map(|argument| self.resolve_type_reference(argument, false))
            .collect::<Vec<_>>();
        let enum_type = self.instantiate_declared_type(name, &arguments, expression.span);
        self.expression_types
            .insert(expression.span, enum_type.clone());
        Some(enum_type)
    }

    fn check_enum_variant(
        &mut self,
        enum_type: &Type,
        variant_name: &Identifier,
        arguments: &[Expression],
        span: SourceSpan,
    ) -> Type {
        if enum_type.is_error() {
            for argument in arguments {
                self.check_expression(argument, None);
            }
            return Type::Error;
        }
        let Some(type_name) = enum_type.nominal_name() else {
            return Type::Error;
        };
        let Some(info) = self.types.get(type_name).cloned() else {
            return Type::Error;
        };
        let Some(variant) = info
            .enum_variants
            .iter()
            .find(|variant| variant.name == variant_name.name)
            .cloned()
        else {
            self.error_with(
                DiagnosticCode::EnumPattern,
                format!("enum '{type_name}' has no variant '{}'", variant_name.name),
                variant_name.span,
            );
            for argument in arguments {
                self.check_expression(argument, None);
            }
            return Type::Error;
        };
        if variant.payload.len() != arguments.len() {
            self.error_with(
                DiagnosticCode::EnumPattern,
                format!(
                    "variant '{}.{}' expects {} payload value(s), found {}",
                    type_name,
                    variant.name,
                    variant.payload.len(),
                    arguments.len()
                ),
                span,
            );
            for argument in arguments {
                self.check_expression(argument, None);
            }
            return Type::Error;
        }
        let substitutions =
            type_substitutions(&info.type_parameters, enum_type.generic_arguments());
        for (index, argument) in arguments.iter().enumerate() {
            let expected = variant
                .payload
                .get(index)
                .map(|ty| substitute_type(ty, &substitutions));
            let Some(expected) = expected else {
                self.check_expression(argument, None);
                continue;
            };
            let actual =
                self.check_expression(argument, is_list_literal(argument).then_some(&expected));
            if actual.is_error()
                || self.record_contract_conversion(&expected, &actual, argument.span)
                || expected.accepts(&actual)
            {
                continue;
            }
            self.error_with(
                DiagnosticCode::EnumPattern,
                format!(
                    "payload {} of variant '{}.{}' expects {expected}, found {actual}",
                    index + 1,
                    type_name,
                    variant.name
                ),
                argument.span,
            );
        }
        enum_type.clone()
    }

    fn check_match_expression(&mut self, matched: &MatchExpression, span: SourceSpan) -> Type {
        let value_type = self.check_expression(&matched.value, None);
        let before = self.scopes.clone();
        let mut branch_states = Vec::new();
        let mut result_type: Option<Type> = None;
        let mut covered_variants = HashSet::new();
        let mut covered_literals = HashSet::new();
        let mut covered_partial_patterns = HashSet::new();
        let mut has_catch_all = false;
        let mut has_invalid_pattern = false;

        for arm in &matched.arms {
            self.scopes = before.clone();
            self.push_scope();
            let already_unreachable = has_catch_all
                || self.patterns_already_exhaustive(
                    &value_type,
                    &covered_variants,
                    &covered_literals,
                );
            if already_unreachable {
                self.error_with(
                    DiagnosticCode::MatchAnalysis,
                    "unreachable match arm; earlier patterns already cover every value",
                    arm.pattern.span,
                );
            }

            let diagnostics_before_pattern = self.diagnostics.len();
            let coverage = self.check_pattern(&arm.pattern, &value_type);
            has_invalid_pattern |= self.diagnostics.len() != diagnostics_before_pattern;
            if !already_unreachable {
                match &coverage {
                    PatternCoverage::CatchAll => has_catch_all = true,
                    PatternCoverage::EnumVariant { index, complete } => {
                        if covered_variants.contains(index) {
                            self.error_with(
                                DiagnosticCode::MatchAnalysis,
                                "unreachable match arm; enum variant was already covered",
                                arm.pattern.span,
                            );
                        } else if *complete {
                            covered_variants.insert(*index);
                        } else if !covered_partial_patterns
                            .insert(pattern_fingerprint(&arm.pattern))
                        {
                            self.error_with(
                                DiagnosticCode::MatchAnalysis,
                                "unreachable match arm; pattern was already covered",
                                arm.pattern.span,
                            );
                        }
                    }
                    PatternCoverage::Literal(key) => {
                        if !covered_literals.insert(key.clone()) {
                            self.error_with(
                                DiagnosticCode::MatchAnalysis,
                                "unreachable match arm; literal was already covered",
                                arm.pattern.span,
                            );
                        }
                    }
                    PatternCoverage::Partial(key) => {
                        if !covered_partial_patterns.insert(key.clone()) {
                            self.error_with(
                                DiagnosticCode::MatchAnalysis,
                                "unreachable match arm; pattern was already covered",
                                arm.pattern.span,
                            );
                        }
                    }
                }
            }

            let arm_type = match &arm.body {
                MatchArmBody::Expression(expression) => self.check_expression(expression, None),
                MatchArmBody::Block(block) => {
                    self.check_block(block, true);
                    Type::Void
                }
            };
            if let Some(previous) = result_type.as_ref() {
                if !previous.is_error() && !arm_type.is_error() {
                    if !previous.accepts(&arm_type) {
                        self.error_with(
                            DiagnosticCode::MatchAnalysis,
                            format!(
                                "match arm has type {arm_type}, but earlier arms have type {previous}"
                            ),
                            arm.span,
                        );
                    }
                }
                if previous.is_error() {
                    result_type = Some(arm_type.clone());
                }
            } else {
                result_type = Some(arm_type);
            }
            self.pop_scope();
            branch_states.push(self.scopes.clone());
        }

        if matched.arms.is_empty() {
            self.error_with(
                DiagnosticCode::Syntax,
                "match expression must contain at least one arm",
                span,
            );
        } else if !has_invalid_pattern
            && !value_type.is_error()
            && !has_catch_all
            && !self.match_is_exhaustive(&value_type, &covered_variants, &covered_literals)
        {
            let missing_variants = self.missing_enum_variants(&value_type, &covered_variants);
            let message = match missing_variants.as_deref() {
                Some([variant]) => format!(
                    "non-exhaustive match for type {value_type}; missing variant: {variant}"
                ),
                Some(variants) if !variants.is_empty() => format!(
                    "non-exhaustive match for type {value_type}; missing variants: {}",
                    variants.join(", ")
                ),
                _ => format!("non-exhaustive match for type {value_type}; add a wildcard arm"),
            };
            self.error_with(DiagnosticCode::MatchAnalysis, message, span);
        }

        // A binding is definitely initialized after a match only when every
        // arm initializes it. Each branch starts from the same incoming state.
        if !branch_states.is_empty() {
            self.scopes = before;
            for (scope_index, scope) in self.scopes.iter_mut().enumerate() {
                for (name, variable) in &mut scope.variables {
                    variable.initialized = branch_states.iter().all(|state| {
                        state
                            .get(scope_index)
                            .and_then(|branch| branch.variables.get(name))
                            .is_some_and(|branch_variable| branch_variable.initialized)
                    });
                }
            }
        }

        result_type.unwrap_or(Type::Error)
    }

    fn check_pattern(&mut self, pattern: &Pattern, expected: &Type) -> PatternCoverage {
        match &pattern.kind {
            PatternKind::Wildcard => PatternCoverage::CatchAll,
            PatternKind::Name(identifier) => {
                if let Some((enum_type, info)) = self.enum_pattern_info(expected) {
                    if let Some((index, variant)) = info
                        .enum_variants
                        .iter()
                        .enumerate()
                        .find(|(_, variant)| variant.name == identifier.name)
                    {
                        if !variant.payload.is_empty() {
                            self.error_with(
                                DiagnosticCode::EnumPattern,
                                format!(
                                    "variant '{}.{}' requires {} payload pattern(s)",
                                    enum_type.nominal_name().unwrap_or("enum"),
                                    variant.name,
                                    variant.payload.len()
                                ),
                                identifier.span,
                            );
                            return PatternCoverage::Partial(pattern_fingerprint(pattern));
                        }
                        return PatternCoverage::EnumVariant {
                            index,
                            complete: true,
                        };
                    }
                    if identifier
                        .name
                        .chars()
                        .next()
                        .is_some_and(char::is_uppercase)
                    {
                        self.error_with(
                            DiagnosticCode::EnumPattern,
                            format!(
                                "enum '{}' has no variant '{}'",
                                enum_type.nominal_name().unwrap_or("<unknown>"),
                                identifier.name
                            ),
                            identifier.span,
                        );
                        return PatternCoverage::Partial(pattern_fingerprint(pattern));
                    }
                }
                self.variable_types
                    .insert(identifier.span, expected.clone());
                self.declare_local(&identifier.name, identifier, expected.clone(), false, true);
                PatternCoverage::CatchAll
            }
            PatternKind::Variant { name, payload } => {
                let Some((enum_type, info)) = self.enum_pattern_info(expected) else {
                    if !expected.is_error() {
                        self.error_with(
                            DiagnosticCode::TypeMismatch,
                            format!("variant pattern requires an enum, found {expected}"),
                            pattern.span,
                        );
                    }
                    return PatternCoverage::Partial(pattern_fingerprint(pattern));
                };
                let Some((index, variant)) = info
                    .enum_variants
                    .iter()
                    .enumerate()
                    .find(|(_, variant)| variant.name == name.name)
                else {
                    self.error_with(
                        DiagnosticCode::EnumPattern,
                        format!(
                            "enum '{}' has no variant '{}'",
                            enum_type.nominal_name().unwrap_or("<unknown>"),
                            name.name
                        ),
                        name.span,
                    );
                    return PatternCoverage::Partial(pattern_fingerprint(pattern));
                };
                if variant.payload.len() != payload.len() {
                    self.error_with(
                        DiagnosticCode::EnumPattern,
                        format!(
                            "variant '{}.{}' expects {} payload pattern(s), found {}",
                            enum_type.nominal_name().unwrap_or("enum"),
                            variant.name,
                            variant.payload.len(),
                            payload.len()
                        ),
                        pattern.span,
                    );
                }
                let substitutions =
                    type_substitutions(&info.type_parameters, enum_type.generic_arguments());
                let mut complete = variant.payload.len() == payload.len();
                for (payload_pattern, payload_type) in payload.iter().zip(&variant.payload) {
                    let payload_type = substitute_type(payload_type, &substitutions);
                    let nested = self.check_pattern(payload_pattern, &payload_type);
                    complete &= nested == PatternCoverage::CatchAll;
                }
                PatternCoverage::EnumVariant { index, complete }
            }
            PatternKind::Literal(literal) => {
                let literal_type = match literal {
                    Literal::Integer(_) => Type::Int,
                    Literal::FloatingPoint(_) => Type::Float,
                    Literal::String(_) => Type::String,
                    Literal::Boolean(_) => Type::Bool,
                };
                match literal {
                    Literal::Integer(value) if value.parse::<i64>().is_err() => self.error_with(
                        DiagnosticCode::TypeMismatch,
                        "integer pattern is outside the supported Int range",
                        pattern.span,
                    ),
                    Literal::FloatingPoint(value) if !matches!(value.parse::<f64>(), Ok(number) if number.is_finite()) => {
                        self.error_with(
                            DiagnosticCode::TypeMismatch,
                            "floating-point pattern is outside the supported Float range",
                            pattern.span,
                        )
                    }
                    _ => {}
                }
                self.require_type(expected, &literal_type, pattern.span, "match pattern");
                PatternCoverage::Literal(pattern_literal_key(literal))
            }
        }
    }

    fn enum_pattern_info(&self, ty: &Type) -> Option<(Type, TypeSymbols)> {
        if !ty.is_enum() {
            return None;
        }
        let name = ty.nominal_name()?;
        Some((ty.clone(), self.types.get(name)?.clone()))
    }

    fn patterns_already_exhaustive(
        &self,
        ty: &Type,
        variants: &HashSet<usize>,
        literals: &HashSet<String>,
    ) -> bool {
        self.match_is_exhaustive(ty, variants, literals)
    }

    fn match_is_exhaustive(
        &self,
        ty: &Type,
        variants: &HashSet<usize>,
        literals: &HashSet<String>,
    ) -> bool {
        if let Some((_, info)) = self.enum_pattern_info(ty) {
            return (0..info.enum_variants.len()).all(|index| variants.contains(&index));
        }
        if *ty == Type::Bool {
            return literals.contains("bool:true") && literals.contains("bool:false");
        }
        false
    }

    fn missing_enum_variants(&self, ty: &Type, covered: &HashSet<usize>) -> Option<Vec<String>> {
        let (enum_type, info) = self.enum_pattern_info(ty)?;
        let name = enum_type.nominal_name().unwrap_or("enum");
        Some(
            info.enum_variants
                .iter()
                .enumerate()
                .filter(|(index, _)| !covered.contains(index))
                .map(|(_, variant)| format!("{name}.{}", variant.name))
                .collect(),
        )
    }

    fn check_expression(&mut self, expression: &Expression, expected: Option<&Type>) -> Type {
        let actual = match &expression.kind {
            ExpressionKind::Identifier(identifier) => self.check_identifier(identifier),
            ExpressionKind::SelfValue => self.current_type.clone().unwrap_or_else(|| {
                self.error(
                    "self is only available inside a class, struct, or trait default method",
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
                if let Some(enum_type) = self.enum_type_from_expression(object) {
                    self.check_enum_variant(&enum_type, member, &[], expression.span)
                } else {
                    let object_type = self.check_expression(object, None);
                    self.member_type(&object_type, member).0
                }
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
            ExpressionKind::Try(operand) => self.check_try_expression(operand, expression.span),
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
            ExpressionKind::Match(matched) => self.check_match_expression(matched, expression.span),
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

    fn check_try_expression(&mut self, operand: &Expression, span: SourceSpan) -> Type {
        let result_type = self.check_expression(operand, None);
        if result_type.is_error() {
            return Type::Error;
        }
        let Some((success_type, error_type)) = self.result_components(&result_type) else {
            self.error_with(
                DiagnosticCode::ResultPropagation,
                format!("cannot apply '?' to {result_type}; expected Result<T, E>"),
                span,
            );
            return Type::Error;
        };
        let Some(enclosing_result_type) = self.current_return_type.clone() else {
            self.error_with(
                DiagnosticCode::ResultPropagation,
                "postfix '?' can only be used inside a function returning Result<T, E>",
                span,
            );
            return Type::Error;
        };
        let Some((_, enclosing_error_type)) = self.result_components(&enclosing_result_type) else {
            self.error_with(
                DiagnosticCode::ResultPropagation,
                format!("cannot propagate an error from a function returning {enclosing_result_type}; the function must return Result<T, E>"),
                span,
            );
            return Type::Error;
        };
        if error_type != enclosing_error_type {
            self.error_with(
                DiagnosticCode::ResultPropagation,
                format!(
                    "cannot propagate error type {error_type}; the enclosing function expects {enclosing_error_type}"
                ),
                span,
            );
            return Type::Error;
        }
        self.result_propagations.insert(
            span,
            ResultPropagation {
                result_type,
                success_type: success_type.clone(),
                error_type,
                enclosing_result_type,
            },
        );
        success_type
    }

    /// Return the success/error payload types only for the ordinary generic
    /// `Result<T, E>` enum with its canonical `Ok(T)` and `Err(E)` variants.
    fn result_components(&self, ty: &Type) -> Option<(Type, Type)> {
        let Type::GenericInstance {
            constructor: GenericTypeConstructor::Enum(name),
            arguments,
        } = ty
        else {
            return None;
        };
        if name != "Result" || arguments.len() != 2 {
            return None;
        }
        let info = self.types.get(name)?;
        if !info.is_enum || info.type_parameters.len() != 2 || info.enum_variants.len() != 2 {
            return None;
        }
        let ok = info
            .enum_variants
            .iter()
            .find(|variant| variant.name == "Ok")?;
        let err = info
            .enum_variants
            .iter()
            .find(|variant| variant.name == "Err")?;
        if ok.payload.as_slice() != [Type::TypeParameter(info.type_parameters[0].clone())]
            || err.payload.as_slice() != [Type::TypeParameter(info.type_parameters[1].clone())]
        {
            return None;
        }
        Some((arguments[0].clone(), arguments[1].clone()))
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
                    "external C functions must be called directly",
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
        if info.is_interface || info.is_trait {
            self.error_with(
                if info.is_interface {
                    DiagnosticCode::InvalidInterface
                } else {
                    DiagnosticCode::InvalidTrait
                },
                format!(
                    "{} '{type_name}' cannot be instantiated",
                    if info.is_interface {
                        "interface"
                    } else {
                        "trait"
                    }
                ),
                type_reference.span,
            );
            for initializer in fields {
                self.check_expression(&initializer.value, None);
            }
            return Type::Error;
        }
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
        if let ExpressionKind::Member { object, member } = &callee.kind {
            if let Some(enum_type) = self.enum_type_from_expression(object) {
                let result = self.check_enum_variant(&enum_type, member, arguments, call_span);
                self.expression_types.insert(callee.span, result.clone());
                return result;
            }
        }
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
                        if info.is_interface || info.is_trait {
                            self.error_with(
                                if info.is_interface {
                                    DiagnosticCode::InvalidInterface
                                } else {
                                    DiagnosticCode::InvalidTrait
                                },
                                format!(
                                    "{} '{}' cannot be instantiated",
                                    if info.is_interface {
                                        "interface"
                                    } else {
                                        "trait"
                                    },
                                    name.name
                                ),
                                call_span,
                            );
                            for argument in arguments {
                                self.check_expression(argument, None);
                            }
                            return Type::Error;
                        }
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
                            true,
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
                        if info.is_interface || info.is_trait {
                            self.error_with(
                                if info.is_interface {
                                    DiagnosticCode::InvalidInterface
                                } else {
                                    DiagnosticCode::InvalidTrait
                                },
                                format!(
                                    "{} '{}' cannot be instantiated",
                                    if info.is_interface {
                                        "interface"
                                    } else {
                                        "trait"
                                    },
                                    identifier.name
                                ),
                                call_span,
                            );
                            for argument in arguments {
                                self.check_expression(argument, None);
                            }
                            return Type::Error;
                        }
                        let generic_parameters = info.type_parameters.clone();
                        let constructor = self.constructor_signature(&identifier.name);
                        let (signature, inferred_arguments) = if generic_parameters.is_empty() {
                            (constructor, None)
                        } else {
                            self.instantiate_signature(
                                &identifier.name,
                                &constructor,
                                &[],
                                arguments,
                                false,
                                call_span,
                            )
                        };
                        let ty = if let Some(type_arguments) = inferred_arguments {
                            self.instantiate_declared_type(
                                &identifier.name,
                                &type_arguments,
                                call_span,
                            )
                        } else {
                            ty
                        };
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
                            true,
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
                        true,
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
                    true,
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
            self.error_with(
                DiagnosticCode::GenericArgument,
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
        self.validate_generic_arguments(
            &info.type_parameters,
            arguments,
            span,
            &format!("type {}", generic_declaration(name, &info.type_parameters)),
        );
        if arguments.is_empty() {
            return info.ty;
        }
        let constructor = match info.ty {
            Type::Class(_) => GenericTypeConstructor::Class(name.to_owned()),
            Type::Struct(_) => GenericTypeConstructor::Struct(name.to_owned()),
            Type::Enum(_) => GenericTypeConstructor::Enum(name.to_owned()),
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
        validate_bounds: bool,
        call_span: SourceSpan,
    ) -> (FunctionType, Option<Vec<Type>>) {
        if signature.type_parameters.is_empty() {
            if !explicit_arguments.is_empty() {
                self.error_with(
                    DiagnosticCode::GenericArgument,
                    format!("'{name}' does not declare generic parameters"),
                    call_span,
                );
            }
            return (signature.clone(), None);
        }

        let mut substitutions = HashMap::new();
        let required_by = format!(
            "fn {}",
            generic_declaration(name, &signature.type_parameters)
        );
        if !explicit_arguments.is_empty() {
            if explicit_arguments.len() != signature.type_parameters.len() {
                self.error_with(
                    DiagnosticCode::GenericArgument,
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
            if explicit_arguments.len() != signature.type_parameters.len() {
                for parameter in signature
                    .type_parameters
                    .iter()
                    .skip(explicit_arguments.len())
                {
                    substitutions.insert(parameter.id, Type::Error);
                }
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
        let unresolved = signature
            .type_parameters
            .iter()
            .filter(|parameter| !substitutions.contains_key(&parameter.id))
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>();
        if !unresolved.is_empty() {
            let unresolved_names = unresolved.join(", ");
            self.error_with(
                DiagnosticCode::GenericInference,
                format!(
                    "cannot infer generic argument(s) {unresolved_names} for '{name}' from the provided arguments; add explicit type arguments"
                ),
                call_span,
            );
            for parameter in &signature.type_parameters {
                substitutions.entry(parameter.id).or_insert(Type::Error);
            }
        }
        let type_arguments = signature
            .type_parameters
            .iter()
            .map(|parameter| substitutions[&parameter.id].clone())
            .collect::<Vec<_>>();
        if validate_bounds {
            self.validate_generic_arguments(
                &signature.type_parameters,
                &type_arguments,
                call_span,
                &required_by,
            );
        }
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
        if let Type::TypeParameter(parameter) = object_type {
            let signatures = parameter
                .bounds
                .iter()
                .filter_map(|bound| self.types.get(bound))
                .filter_map(|contract| contract.methods.get(&member.name))
                .cloned()
                .collect::<Vec<_>>();
            if let Some(signature) = signatures.first() {
                if signatures.iter().any(|candidate| candidate != signature) {
                    self.error_with(
                        DiagnosticCode::InvalidMember,
                        format!(
                            "method '{}' is ambiguous across bounds on type parameter '{}'",
                            member.name, parameter.name
                        ),
                        member.span,
                    );
                    return (Type::Error, false);
                }
                return (Type::Function(signature.clone()), false);
            }
            self.error_with(
                DiagnosticCode::InvalidMember,
                format!(
                    "type parameter '{}' has no method '{}'; add a bound that declares it",
                    parameter.name, member.name
                ),
                member.span,
            );
            return (Type::Error, false);
        }
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
        if self.record_contract_conversion(expected, actual, span) || expected.accepts(actual) {
            return;
        }
        self.error_with(
            DiagnosticCode::TypeMismatch,
            format!("expected {expected}, found {actual} ({context})"),
            span,
        );
    }

    fn record_contract_conversion(
        &mut self,
        expected: &Type,
        actual: &Type,
        span: SourceSpan,
    ) -> bool {
        let (contract_name, is_interface) = match expected {
            Type::Interface(name) => (name.as_str(), true),
            Type::Trait(name) => (name.as_str(), false),
            _ => return false,
        };
        let Some(class_name) = actual.nominal_name() else {
            return false;
        };
        let source_is_class = matches!(
            actual,
            Type::Class(_)
                | Type::GenericInstance {
                    constructor: GenericTypeConstructor::Class(_),
                    ..
                }
        );
        let source_is_struct = actual.is_struct();
        if (is_interface && !source_is_class)
            || (!is_interface && !source_is_class && !source_is_struct)
        {
            return false;
        }
        let conforms = self.types.get(class_name).is_some_and(|info| {
            let implementations = if is_interface {
                &info.implemented_interfaces
            } else {
                &info.implemented_traits
            };
            implementations.iter().any(|name| name == contract_name)
        });
        if !conforms
            && self
                .invalid_contract_implementations
                .contains(&(class_name.to_owned(), contract_name.to_owned()))
        {
            // The implementation declaration already explains why this value
            // is not conforming; avoid repeating the same failure as a type
            // mismatch at every use site.
            return true;
        }
        if conforms {
            self.contract_conversions.insert(span, expected.clone());
        }
        conforms
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

    fn error_with_note(
        &mut self,
        code: DiagnosticCode,
        message: impl Into<String>,
        span: SourceSpan,
        note: impl Into<String>,
    ) {
        self.diagnostics
            .push(Diagnostic::at_source(code, message, self.source, span).with_note(note));
    }
}

fn generic_declaration(name: &str, parameters: &[GenericTypeParameter]) -> String {
    if parameters.is_empty() {
        return name.to_owned();
    }
    let parameters = parameters
        .iter()
        .map(|parameter| {
            if parameter.bounds.is_empty() {
                parameter.name.clone()
            } else {
                format!("{}: {}", parameter.name, parameter.bounds.join(" + "))
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name}<{parameters}>")
}

fn function_signature(signature: &FunctionType) -> String {
    Type::Function(signature.clone()).to_string()
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

fn pattern_literal_key(literal: &Literal) -> String {
    match literal {
        Literal::Integer(value) => {
            let normalized = value
                .parse::<i64>()
                .map_or_else(|_| value.clone(), |number| number.to_string());
            format!("int:{normalized}")
        }
        Literal::FloatingPoint(value) => match value.parse::<f64>() {
            Ok(number) => format!("float:{:016x}", number.to_bits()),
            Err(_) => format!("float:{value}"),
        },
        Literal::String(value) => format!("string:{value}"),
        Literal::Boolean(value) => format!("bool:{value}"),
    }
}

fn pattern_fingerprint(pattern: &Pattern) -> String {
    match &pattern.kind {
        PatternKind::Wildcard => "_".to_owned(),
        PatternKind::Name(name) => format!("name:{}", name.name),
        PatternKind::Variant { name, payload } => format!(
            "variant:{}({})",
            name.name,
            payload
                .iter()
                .map(pattern_fingerprint)
                .collect::<Vec<_>>()
                .join(",")
        ),
        PatternKind::Literal(literal) => pattern_literal_key(literal),
    }
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

fn is_try_expression(expression: &Expression) -> bool {
    matches!(ungroup_kind(&expression.kind), ExpressionKind::Try(_))
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
    use crate::ast::{Declaration, ExpressionKind, StatementKind};
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
    fn checks_interface_conformance_and_class_to_interface_value_conversions() {
        let result = analyze_text(
            r#"interface Drawable {
    fn draw(prefix: String) -> String
}
interface Clickable {
    fn click() -> Bool
}
class Widget implements Drawable, Clickable {
    name: String
    init(name: String) { self.name = name }
    fn draw(prefix: String) -> String { return prefix + self.name }
    fn click() -> Bool { return true }
}
class Holder {
    value: Drawable
    init(value: Drawable) { self.value = value }
    fn get() -> Drawable { return self.value }
}
fn render(value: Drawable) -> Drawable {
    println(value.draw("item: "))
    return value
}
fn main() {
    let widget = Widget("button")
    let drawable: Drawable = widget
    let result = render(widget)
    let holder = Holder(widget)
    let values: List<Drawable> = [widget, result, holder.get()]
    values.add(drawable)
    values[0] = holder.get()
    println(values[0].draw("list: "))
}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        let widget = result
            .typed_program
            .symbols
            .type_symbols("Widget")
            .expect("class symbols should be registered");
        assert_eq!(widget.implemented_interfaces, ["Drawable", "Clickable"]);
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 1)),
            Some(&Type::Interface("Drawable".to_owned()))
        );
        let main = function(&result, "main");
        let StatementKind::Variable(view) = &main.body.statements[1].kind else {
            panic!("expected interface-typed local");
        };
        let value = view.initializer.as_ref().expect("interface initializer");
        assert!(result
            .typed_program
            .contract_conversion(value.span)
            .is_some_and(|ty| ty == &Type::Interface("Drawable".to_owned())));
    }

    #[test]
    fn diagnoses_invalid_interface_implementations_and_instantiation() {
        let result = analyze_text(
            r#"interface Named { fn name() -> String }
class Missing implements Named {}
class Wrong implements Named { fn name() -> Int { return 1 } }
class Repeated implements Named, Named { fn name() -> String { return "x" } }
class Unknown implements Absent {}
class NotAnInterface {}
class Invalid implements NotAnInterface {}
fn main() {
    let value = Named()
}"#,
        );
        assert!(has_message(
            &result,
            "is missing method 'name' required by interface 'Named'"
        ));
        assert!(has_message(&result, "does not match interface 'Named'"));
        assert!(has_message(&result, "listed more than once"));
        assert!(has_message(&result, "unknown interface 'Absent'"));
        assert!(has_message(&result, "is not an interface"));
        assert!(has_message(
            &result,
            "interface 'Named' cannot be instantiated"
        ));
    }

    #[test]
    fn rejects_struct_interface_implementation() {
        let result = analyze_text(
            r#"interface Named { fn name() -> String }
struct Point implements Named { x: Int }
fn main() {}"#,
        );
        assert!(has_message(
            &result,
            "only classes can implement interfaces in v0.2"
        ));
    }

    #[test]
    fn checks_trait_defaults_overrides_multiple_traits_and_trait_values() {
        let result = analyze_text(
            r#"trait Named {
    fn name() -> String
    fn describe() { println(self.name()) }
}
trait Tagged {
    fn tag() -> String
    fn label() { println(self.tag()) }
}
class User implements Named, Tagged {
    value: String
    init(value: String) { self.value = value }
    fn name() -> String { return self.value }
    fn tag() -> String { return "user" }
    fn describe() { println("custom: " + self.name()) }
}
struct Point implements Named {
    value: String
    fn name() -> String { return self.value }
}
fn show(value: Named) { value.describe() }
fn main() {
    let user = User("Ada")
    let named: Named = user
    show(user)
    named.describe()
    let point = Point("origin")
    let pointName: Named = point
    pointName.describe()
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        let user = result
            .typed_program
            .symbols
            .type_symbols("User")
            .expect("class symbols");
        assert_eq!(user.implemented_traits, ["Named", "Tagged"]);
        assert_eq!(
            user.default_trait_methods.get("label"),
            Some(&"Tagged".to_owned())
        );
        let point = result
            .typed_program
            .symbols
            .type_symbols("Point")
            .expect("struct symbols");
        assert_eq!(point.implemented_traits, ["Named"]);
        assert_eq!(
            point.default_trait_methods.get("describe"),
            Some(&"Named".to_owned())
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 1)),
            Some(&Type::Trait("Named".to_owned()))
        );
    }

    #[test]
    fn diagnoses_trait_conflicts_missing_methods_and_invalid_declarations() {
        let result = analyze_text(
            r#"trait First { fn speak() { println("first") } }
trait Second { fn speak() { println("second") } }
trait Required { fn run(value: Int) -> Int }
class Ambiguous implements First, Second {}
class Missing implements Required {}
class Invalid implements Absent {}
class NotATrait {}
class InvalidTarget implements NotATrait {}
class Repeated implements Required, Required { fn run(value: Int) -> Int { return value } }
class WrongSignature implements Required { fn run(value: String) -> Int { return 1 } }
struct AlsoMissing implements Required { value: Int }
fn main() {
    let value = Required()
}"#,
        );
        assert!(has_message(&result, "conflicting default method 'speak'"));
        assert!(has_message(
            &result,
            "is missing method 'run' required by trait 'Required'"
        ));
        assert!(has_message(&result, "unknown interface 'Absent' or trait"));
        assert!(has_message(&result, "is not an interface or trait"));
        assert!(has_message(&result, "listed more than once in implements"));
        assert!(has_message(&result, "does not match trait 'Required'"));
        assert!(has_message(
            &result,
            "trait 'Required' cannot be instantiated"
        ));
    }

    #[test]
    fn explicit_trait_method_resolves_conflicting_defaults() {
        let result = analyze_text(
            r#"trait First { fn speak() { println("first") } }
trait Second { fn speak() { println("second") } }
class Combined implements First, Second {
    fn speak() { println("combined") }
}
fn main() { let value: First = Combined(); value.speak() }"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
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
            "methods on structs are supported only when they implement a trait"
        ));
        assert!(has_message(
            &result,
            "structs do not support init constructors"
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
    fn infers_generic_class_and_struct_construction_from_constructor_arguments() {
        let result = analyze_text(
            r#"class Box<T> {
    value: T
    init(value: T) { self.value = value }
}
struct Pair<A, B> {
    first: A
    second: B
}
fn main() {
    let box = Box(42)
    let pair = Pair("one", 1.0)
    let nested = Box([1, 2, 3])
}"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
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
                GenericTypeConstructor::Struct("Pair".to_owned()),
                vec![Type::String, Type::Float]
            ))
        );
        assert_eq!(
            result
                .typed_program
                .variable_type(variable(&result, "main", 2)),
            Some(&Type::generic_instance(
                GenericTypeConstructor::Class("Box".to_owned()),
                vec![Type::list(Type::Int)]
            ))
        );
    }

    #[test]
    fn requires_explicit_arguments_when_generic_construction_is_ambiguous() {
        let result = analyze_text("class Empty<T> {}\nfn main() { let empty = Empty() }");
        assert!(has_message(
            &result,
            "cannot infer generic argument(s) T for 'Empty' from the provided arguments"
        ));
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
            "cannot infer generic argument(s) T for 'empty' from the provided arguments"
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
    fn checks_function_class_and_struct_bounds_and_bound_methods() {
        let result = analyze_text(
            r#"trait Named { fn name() -> String }
trait Described { fn description() -> String { return "default" } }
interface Drawable { fn draw() }
class User implements Named, Described, Drawable {
    value: String
    init(value: String) { self.value = value }
    fn name() -> String { return self.value }
    fn draw() { println(self.value) }
}
fn acceptRepository(value: Repository<User>) {}
fn acceptBounded<T: Named>(value: Repository<T>) {}
class Catalog { repository: Repository<User> }
fn summarize<T: Named + Described>(value: T) -> String {
    return value.name() + ":" + value.description()
}
fn render<T: Drawable>(value: T) { value.draw() }
class Repository<T: Named> {
    value: T
    init(value: T) { self.value = value }
    fn name() -> String { return self.value.name() }
}
struct NamedHolder<T: Named> implements Named {
    value: T
    fn name() -> String { return self.value.name() }
}
fn main() {
    let user = User("Ada")
    let summary = summarize(user)
    render(user)
    let repository = Repository<User>(user)
    let repositoryName = repository.name()
    let holder = NamedHolder<User> { value: user }
    let holderName = holder.name()
}"#,
        );

        assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
        let Some(super::GlobalSymbol::Function(signature)) =
            result.typed_program.symbols.global("summarize")
        else {
            panic!("bounded generic function signature should be registered");
        };
        assert_eq!(signature.type_parameters[0].bounds, ["Named", "Described"]);
        assert_eq!(
            result
                .typed_program
                .symbols
                .type_symbols("Repository")
                .unwrap()
                .type_parameters[0]
                .bounds,
            ["Named"]
        );
        assert_eq!(
            result
                .typed_program
                .symbols
                .type_symbols("NamedHolder")
                .unwrap()
                .type_parameters[0]
                .bounds,
            ["Named"]
        );
    }

    #[test]
    fn rejects_unsatisfied_function_class_struct_and_interface_bounds() {
        let result = analyze_text(
            r#"trait Named { fn name() -> String }
trait Tagged { fn tag() -> String }
interface Drawable { fn draw() }
class User implements Named {
    fn name() -> String { return "user" }
}
fn show<T: Named>(value: T) -> String { return value.name() }
fn both<T: Named + Tagged>(value: T) { println(value.name()); println(value.tag()) }
fn draw<T: Drawable>(value: T) { value.draw() }
class Repository<T: Named> { value: T }
struct Holder<T: Named> { value: T }
fn unbounded<T>(value: T) { value.name() }
fn unboundedRepository<T>(value: Repository<T>) {}
fn main() {
    show(42)
    both(User())
    draw(User())
    let repository = Repository<Int>(1)
    let holder = Holder<Int> { value: 1 }
    unbounded(1)
}"#,
        );

        assert!(has_message(
            &result,
            "type Int does not satisfy trait Named"
        ));
        assert!(has_message(
            &result,
            "type User does not satisfy trait Tagged"
        ));
        assert!(has_message(
            &result,
            "type User does not satisfy interface Drawable"
        ));
        assert!(has_message(
            &result,
            "type parameter 'T' has no method 'name'; add a bound that declares it"
        ));
        assert!(has_message(&result, "type T does not satisfy trait Named"));
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic
                    .message()
                    .contains("type Int does not satisfy trait Named"))
                .count(),
            3,
            "the function argument and class/struct generic instances should be checked"
        );
    }

    #[test]
    fn v02_diagnostics_keep_specific_codes_locations_and_secondary_context() {
        let cases = [
            (
                "fn identity<T>(value: T) -> T { return value }\nfn main() { let item = identity<Int, String>(1) }",
                DiagnosticCode::GenericArgument,
                "'identity' expects 1 generic argument(s), found 2",
            ),
            (
                "fn empty<T>() -> List<T> { return [] }\nfn main() { let values = empty() }",
                DiagnosticCode::GenericInference,
                "cannot infer generic argument(s) T for 'empty'",
            ),
            (
                "trait Printable { fn print() }\nfn show<T: Printable>(value: T) {}\nfn main() { show(1) }",
                DiagnosticCode::GenericConstraint,
                "type Int does not satisfy trait Printable",
            ),
            (
                "interface Drawable { fn draw(label: String) -> Bool }\nclass Button implements Drawable {}\nfn main() {}",
                DiagnosticCode::InvalidInterface,
                "type 'Button' is missing method 'draw' required by interface 'Drawable'",
            ),
            (
                "enum State { Ready Waiting }\nfn main() { let state = State.Ready; match state { Ready => println(1) } }",
                DiagnosticCode::MatchAnalysis,
                "non-exhaustive match for type State; missing variant: State.Waiting",
            ),
            (
                "fn bad(value: Int) -> Int { return value? }\nfn main() {}",
                DiagnosticCode::ResultPropagation,
                "cannot apply '?' to Int; expected Result<T, E>",
            ),
        ];

        for (program, expected_code, expected_message) in cases {
            let result = analyze_text(program);
            let diagnostic = result
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code() == expected_code)
                .unwrap_or_else(|| panic!("missing {expected_code:?}: {:?}", messages(&result)));
            let rendered = diagnostic.to_string();
            assert!(rendered.contains(expected_message), "{rendered}");
            assert!(rendered.contains("semantic.prnc:"), "{rendered}");
            assert!(
                rendered.contains('\n') && rendered.contains('^'),
                "{rendered}"
            );
            if expected_code == DiagnosticCode::GenericConstraint {
                assert!(
                    rendered.contains("= note: required by fn show<T: Printable>"),
                    "{rendered}"
                );
            }
            assert!(!rendered.contains("LLVM"), "{rendered}");
            assert!(!rendered.contains("monomorph"), "{rendered}");
        }
    }

    #[test]
    fn validates_bound_declarations_and_compatible_method_sets() {
        let invalid_bounds = analyze_text(
            "trait Named { fn name() -> String }\nfn unknown<T: Missing>(value: T) {}\nfn concrete<T: Int>(value: T) {}\nfn duplicate<T: Named + Named>(value: T) {}\nfn generic<T: Named<Int>>(value: T) {}\nfn main() {}",
        );
        assert!(has_message(
            &invalid_bounds,
            "unknown type 'Missing' in generic bound"
        ));
        assert!(has_message(
            &invalid_bounds,
            "generic bound 'Int' must name an interface or trait"
        ));
        assert!(has_message(&invalid_bounds, "duplicate bound 'Named'"));
        assert!(has_message(
            &invalid_bounds,
            "generic bounds cannot have type arguments in v0.2"
        ));

        let conflicting_methods = analyze_text(
            r#"trait StringValue { fn value() -> String }
trait IntValue { fn value() -> Int }
fn use<T: StringValue + IntValue>(item: T) { item.value() }
fn main() {}"#,
        );
        assert!(has_message(
            &conflicting_methods,
            "method 'value' is ambiguous across bounds on type parameter 'T'"
        ));

        let shared_method = analyze_text(
            r#"trait Named { fn name() -> String }
trait AlsoNamed { fn name() -> String }
fn use<T: Named + AlsoNamed>(item: T) -> String { return item.name() }
fn main() {}"#,
        );
        assert!(
            shared_method.diagnostics.is_empty(),
            "identical methods from multiple bounds are unambiguous: {:?}",
            messages(&shared_method)
        );
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
    fn checks_simple_payload_generic_and_nested_enum_values() {
        let result = analyze_text(
            r#"struct Point { x: Int }
class User { name: String }
enum Direction { North South }
enum NetworkState { Connected Failed(String) }
enum Maybe<T> { Some(T) None }
enum Result<T, E> { Ok(T) Err(E) }
fn pass(value: Result<Int, String>) -> Result<Int, String> { return value }
fn main() {
    let direction = Direction.North
    let state = NetworkState.Failed("offline")
    let maybe = Maybe<Point>.Some(Point(3))
    let result = pass(Result<Int, String>.Ok(42))
    let absent = Maybe<Int>.None
    let nested: List<Maybe<Result<Int, String>>> = [Maybe<Result<Int, String>>.Some(result)]
    let held = Maybe<User>.Some(User("Mira"))
}"#,
        );
        assert!(
            result.diagnostics.is_empty(),
            "enum values should resolve through generic payload types: {:?}",
            messages(&result)
        );
        let maybe = result
            .typed_program
            .symbols
            .type_symbols("Maybe")
            .expect("enum type should be registered");
        assert!(maybe.is_enum);
        assert_eq!(maybe.enum_variants.len(), 2);
        assert_eq!(
            maybe.enum_variants[0].payload,
            [Type::TypeParameter(maybe.type_parameters[0].clone())]
        );
    }

    #[test]
    fn checks_result_propagation_and_records_types_for_generic_lowering() {
        let result = analyze_text(
            r#"enum Result<T, E> { Ok(T) Err(E) }
fn read() -> Result<Int, String> { return Result<Int, String>.Ok(4) }
fn apply<T>(value: Result<T, String>) -> Result<T, String> {
    let item = value?
    return Result<T, String>.Ok(item)
}
fn direct(value: Result<Int, String>) -> Result<Int, String> { return (value?) }
fn main() { let result = apply<Int>(read()) }"#,
        );
        assert!(
            result.diagnostics.is_empty(),
            "valid propagation should type check: {:?}",
            messages(&result)
        );

        let apply = function(&result, "apply");
        let StatementKind::Variable(variable) = &apply.body.statements[0].kind else {
            panic!("expected propagated local declaration");
        };
        let expression = variable.initializer.as_ref().expect("initializer");
        let ExpressionKind::Try(_) = expression.kind else {
            panic!("initializer should contain a Try expression");
        };
        assert_eq!(
            result.typed_program.expression_type(expression),
            Some(&Type::TypeParameter(
                result
                    .typed_program
                    .symbols
                    .global("apply")
                    .and_then(|symbol| match symbol {
                        super::GlobalSymbol::Function(signature) => {
                            signature.type_parameters.first().cloned()
                        }
                        _ => None,
                    })
                    .expect("generic parameter")
            ))
        );
        let propagation = result
            .typed_program
            .result_propagation(expression.span)
            .expect("typed propagation data");
        assert!(matches!(
            &propagation.success_type,
            Type::TypeParameter(parameter) if parameter.name == "T"
        ));
        assert_eq!(propagation.error_type, Type::String);

        let direct = function(&result, "direct");
        let StatementKind::Return(Some(returned)) = &direct.body.statements[0].kind else {
            panic!("expected return statement");
        };
        assert!(result
            .typed_program
            .return_result_wrapping(returned.span)
            .is_some());
    }

    #[test]
    fn diagnoses_invalid_result_propagation_contexts_and_error_types() {
        let result = analyze_text(
            r#"enum Result<T, E> { Ok(T) Err(E) }
fn nonResult(value: Int) -> Int { return value? }
fn wrongReturn(value: Result<Int, String>) -> Int { return value? }
fn wrongError(value: Result<Int, Int>) -> Result<Int, String> { let item = value?; return Result<Int, String>.Ok(item) }
fn nonResultOperand() -> Result<Int, String> { return 5? }
fn main() {}"#,
        );
        assert!(has_message(
            &result,
            "the function must return Result<T, E>"
        ));
        assert!(has_message(
            &result,
            "cannot propagate error type Int; the enclosing function expects String"
        ));
        assert!(has_message(
            &result,
            "cannot apply '?' to Int; expected Result<T, E>"
        ));
    }

    #[test]
    fn rejects_unknown_enum_variants_payload_errors_duplicates_and_recursive_layouts() {
        let result = analyze_text(
            r#"enum Token { End Value(Int) Value(String) }
enum Bad { Nested(Bad) }
enum Maybe<T> { Some(T) None }
fn main() {
    let unknown = Token.Missing
    let missing = Token.Value
    let extra = Token.End(1)
    let wrong = Token.Value("text")
    let bad_arity = Maybe<Int>.Some()
}"#,
        );
        assert!(has_message(&result, "duplicate variant 'Value'"));
        assert!(has_message(&result, "has no variant 'Missing'"));
        assert!(has_message(&result, "expects 1 payload value(s), found 0"));
        assert!(has_message(&result, "expects 0 payload value(s), found 1"));
        assert!(
            has_message(&result, "expects Int, found String"),
            "{:?}",
            messages(&result)
        );
        assert!(has_message(
            &result,
            "recursive enum payloads do not have a finite value layout"
        ));
    }

    #[test]
    fn validates_generic_enum_arity_and_unknown_payload_types() {
        let result = analyze_text(
            "enum Maybe<T> { Some(T) None }\nfn main() { let wrong = Maybe<Int, String>.None; let unknown = Maybe<Missing>.None }",
        );
        assert!(has_message(
            &result,
            "type 'Maybe' expects 1 generic argument(s), found 2"
        ));
        assert!(has_message(&result, "unknown type 'Missing'"));
    }

    #[test]
    fn checks_enum_match_payload_bindings_expression_types_and_arm_scopes() {
        let source = r#"enum Result<T, E> { Ok(T) Err(E) }
enum State { Online Offline }
fn message(result: Result<Int, String>) -> String {
    return match result {
        Ok(value) => "received"
        Err(error) => error
    }
}
fn main() {
    let state = State.Online
    let label = match state {
        Online => "online"
        Offline => "offline"
    }
    println(message(Result<Int, String>.Err("failure")))
    println(label)
}"#;
        for extension in ["prnc", "princi"] {
            let result = analyze_source(&format!("match.{extension}"), source);
            assert!(result.diagnostics.is_empty(), "{:?}", messages(&result));
            assert_eq!(
                result
                    .typed_program
                    .variable_type(variable(&result, "main", 1)),
                Some(&Type::String)
            );
        }

        let leaked = analyze_text(
            r#"enum Maybe<T> { Some(T) None }
fn main() {
    let value = Maybe<Int>.Some(1)
    match value { Some(item) => println(item) None => println("none") }
    println(item)
}"#,
        );
        assert!(has_message(&leaked, "undefined identifier 'item'"));
    }

    #[test]
    fn rejects_non_exhaustive_unreachable_invalid_and_incompatible_match_arms() {
        let result = analyze_text(
            r#"enum State { Ready Failed(String) }
enum Maybe<T> { Some(T) None }
fn main() {
    let state = State.Ready
    match state { Ready => println("ready") }
    match state {
        Missing(value) => println(value)
        Failed => println("missing payload")
        _ => println("fallback")
    }
    match true { true => println("yes") true => println("again") false => println("no") }
    let invalid = match state { Ready => 1 Failed(_) => "failed" }
    let bad = Maybe<Int>.Some(1)
    match bad { Some(left, right) => println(left) None => println("none") }
}"#,
        );
        assert!(has_message(
            &result,
            "non-exhaustive match for type State; missing variant: State.Failed"
        ));
        assert!(has_message(
            &result,
            "enum 'State' has no variant 'Missing'"
        ));
        assert!(has_message(
            &result,
            "variant 'State.Failed' requires 1 payload pattern(s)"
        ));
        assert!(has_message(&result, "unreachable match arm"));
        assert!(has_message(
            &result,
            "match arm has type String, but earlier arms have type Int"
        ));
        assert!(has_message(
            &result,
            "expects 1 payload pattern(s), found 2"
        ));
    }

    #[test]
    fn supports_boolean_and_literal_match_coverage() {
        let valid = analyze_text(
            "fn main() { let ready = true; let text = match ready { true => \"yes\" false => \"no\" }; let number = match 2 { 1 => \"one\" 2 => \"two\" _ => \"other\" } }",
        );
        assert!(valid.diagnostics.is_empty(), "{:?}", messages(&valid));
        let invalid = analyze_text(
            "fn main() { let ready = true; match ready { true => println(1) _ => println(2) false => println(3) } }",
        );
        assert!(has_message(&invalid, "unreachable match arm"));
    }

    #[test]
    fn reports_uncovered_variants_for_concrete_generic_enum_instances() {
        let result = analyze_text(
            r#"enum Boolish { Yes No }
enum Result<T, E> { Ok(T) Err(E) }
fn complete(value: Boolish) {
    match value { Yes => println("yes") _ => println("no") }
}
fn generic_complete(value: Result<Int, String>) {
    match value { Ok(_) => println("ok") Err(_) => println("error") }
}
fn generic_missing(value: Result<String, Int>) {
    match value { Ok(_) => println("ok") }
}
fn main() {
    let value = Boolish.Yes
    match value { Yes => println("yes") }
}"#,
        );

        assert!(has_message(
            &result,
            "non-exhaustive match for type Result<String, Int>; missing variant: Result.Err"
        ));
        assert!(has_message(
            &result,
            "non-exhaustive match for type Boolish; missing variant: Boolish.No"
        ));
        assert_eq!(result.diagnostics.len(), 2, "{:?}", messages(&result));
    }

    #[test]
    fn catch_all_makes_later_enum_arms_unreachable_and_literals_need_default() {
        let result = analyze_text(
            r#"enum Result<T, E> { Ok(T) Err(E) }
fn main() {
    let result = Result<Int, String>.Ok(1)
    match result {
        _ => println("handled")
        Ok(value) => println(value)
    }
    match 1 { 1 => println("one") }
}"#,
        );

        assert!(has_message(&result, "unreachable match arm"));
        assert!(has_message(
            &result,
            "non-exhaustive match for type Int; add a wildcard arm"
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

        assert!(has_message(&result, "only extern \"C\" is supported"));
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
            "external C functions must be called directly"
        ));
    }
}
