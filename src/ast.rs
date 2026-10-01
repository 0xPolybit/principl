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
    Import(ImportDeclaration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDeclaration {
    pub path: Vec<Identifier>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDeclaration {
    pub name: Identifier,
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
    pub members: Vec<ClassMember>,
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
    Construction {
        type_name: Identifier,
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
