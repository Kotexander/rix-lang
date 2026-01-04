use super::UniqueSymbol;
use crate::lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub(super) u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Identifier(UniqueSymbol),
    Integer(i64),
    String(UniqueSymbol),

    Group(ExprId),
    BinOp { op: BinOp, lhs: ExprId, rhs: ExprId },
    UniOp { op: UniOp, expr: ExprId },
    Index { base: ExprId, index: ExprId },
    Call { callee: ExprId, args: Vec<ExprId> },

    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: lexer::Span,
}
impl Expr {
    pub fn new(kind: ExprKind, span: lexer::Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Mul,
    Div,
    Rem,

    Add,
    Sub,

    Shl,
    Shr,

    Lt,
    Gt,
    Le,
    Ge,

    Eq,
    Ne,

    BitAnd,
    BitXor,
    BitOr,

    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniOp {
    Neg,
    Not,
    Ref,
    Deref,
}
