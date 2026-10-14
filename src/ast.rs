use crate::source::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Function(FunctionDeclaration),
    Class(TypeDeclaration),
    Struct(TypeDeclaration),
    Enum(EnumDeclaration),
    Interface(InterfaceDeclaration),
    Trait(TraitDeclaration),
    Import(ImportDeclaration),
    ExternBlock(ExternBlockDeclaration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDeclaration {
    pub name: Identifier,
    pub type_parameters: Vec<TypeParameterDeclaration>,
    pub variants: Vec<EnumVariantDeclaration>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantDeclaration {
    pub name: Identifier,
    pub payload: Vec<TypeReference>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
    pub name: String,
    pub arguments: Vec<TypeReference>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParameterDeclaration {
    pub name: Identifier,
    pub bounds: Vec<TypeReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDeclaration {
    pub path: Vec<Identifier>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternBlockDeclaration {
    pub abi: String,
    pub abi_span: SourceSpan,
    pub functions: Vec<ExternalFunctionDeclaration>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalFunctionDeclaration {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeReference>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub type_parameters: Vec<TypeParameterDeclaration>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeReference>,
    pub body: Block,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Identifier,
    pub type_reference: TypeReference,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDeclarationKind {
    Class,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDeclaration {
    pub kind: TypeDeclarationKind,
    pub name: Identifier,
    pub type_parameters: Vec<TypeParameterDeclaration>,
    pub interfaces: Vec<TypeReference>,
    pub members: Vec<ClassMember>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDeclaration {
    pub name: Identifier,
    pub methods: Vec<InterfaceMethodDeclaration>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceMethodDeclaration {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeReference>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDeclaration {
    pub name: Identifier,
    pub methods: Vec<TraitMethodDeclaration>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitMethodDeclaration {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeReference>,
    /// A body marks this method as a default implementation.
    pub body: Option<Block>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassMember {
    Field(FieldDeclaration),
    Method(FunctionDeclaration),
    Initializer(InitializerDeclaration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDeclaration {
    pub name: Identifier,
    pub type_reference: TypeReference,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializerDeclaration {
    pub parameters: Vec<Parameter>,
    pub body: Block,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Variable(VariableDeclaration),
    Assignment(Assignment),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Return(Option<Expression>),
    Expression(Expression),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDeclaration {
    pub mutable: bool,
    pub name: Identifier,
    pub type_reference: Option<TypeReference>,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub target: Expression,
    pub operator: AssignmentOperator,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Block,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStatement {
    pub variable: Identifier,
    pub range: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchExpression {
    pub value: Box<Expression>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchArmBody,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchArmBody {
    Expression(Expression),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternKind {
    Wildcard,
    /// A bare name is resolved as a nullary enum variant when it names one;
    /// otherwise it introduces an immutable arm-local binding.
    Name(Identifier),
    Variant {
        name: Identifier,
        payload: Vec<Pattern>,
    },
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind {
    Identifier(Identifier),
    SelfValue,
    Literal(Literal),
    List(Vec<Expression>),
    GenericReference {
        name: Identifier,
        arguments: Vec<TypeReference>,
    },
    Construction {
        type_reference: TypeReference,
        fields: Vec<FieldInitializer>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        member: Identifier,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },
    Match(MatchExpression),
    Group(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInitializer {
    pub name: Identifier,
    pub value: Expression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Integer(String),
    FloatingPoint(String),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}
