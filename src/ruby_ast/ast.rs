#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Expression>,
}

impl Program {
    pub fn is_format_candidate(&self) -> bool {
        self.statements
            .first()
            .is_some_and(Expression::is_format_candidate)
            && self.statements.len() == 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Array(Vec<Expression>),
    Assignment(Assignment),
    Binary(Binary),
    Call(Call),
    ConstantPath(ConstantPath),
    Grouped(Box<Expression>),
    Hash(Vec<HashPair>),
    Identifier(String),
    Index(Index),
    Literal(String),
    MemberAccess(MemberAccess),
    Modifier(Modifier),
    PercentLiteral(String),
    Range(Range),
    String(String),
    Symbol(String),
    Ternary(Ternary),
    Unary(Unary),
}

impl Expression {
    pub fn is_format_candidate(&self) -> bool {
        !matches!(
            self,
            Self::Assignment(_) | Self::Modifier(_) | Self::Ternary(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub target: String,
    pub operator: AssignmentOperator,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentOperator {
    Add,
    And,
    Assign,
    Divide,
    Modulo,
    Multiply,
    Or,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    And,
    Divide,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Modulo,
    Multiply,
    NotEqual,
    Or,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unary {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberAccess {
    pub receiver: Box<Expression>,
    pub operator: MemberOperator,
    pub member: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberOperator {
    Dot,
    SafeNavigation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub receiver: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantPath {
    pub parts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    pub expression: Box<Expression>,
    pub keyword: ModifierKeyword,
    pub condition: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierKeyword {
    If,
    Unless,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ternary {
    pub condition: Box<Expression>,
    pub truthy: Box<Expression>,
    pub falsy: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub start: Box<Expression>,
    pub operator: RangeOperator,
    pub end: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeOperator {
    Exclusive,
    Inclusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub callee: String,
    pub arguments: Vec<Argument>,
    pub style: CallStyle,
    pub block: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallStyle {
    Command,
    Parenthesized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument {
    Block(Expression),
    HashPair { key: Expression, value: Expression },
    Keyword { name: String, value: Expression },
    KeywordSplat(Expression),
    Positional(Expression),
    Splat(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashPair {
    KeyValue { key: HashKey, value: Expression },
    KeywordSplat(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashKey {
    Label(String),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub params: Vec<String>,
}
