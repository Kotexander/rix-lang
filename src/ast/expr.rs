use super::arena::{Arena, ArenaId};
use super::symbols::SymbolId;
use crate::lexer;

pub type ExprArena = Arena<Expr>;
pub type ExprId = ArenaId<Expr>;

impl ExprArena {
    pub fn add(&mut self, kind: ExprKind, span: lexer::Span) -> ExprId {
        self.alloc(Expr::new(kind, span))
    }
}

pub type ArgList = Vec<ExprId>;
pub type ArgListArena = Arena<ArgList>;
pub type ArgListId = ArenaId<ArgList>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprKind {
    Identifier(SymbolId),
    /// TODO: dont use i64
    Integer(i64),
    String(SymbolId),

    Group(ExprId),
    BinOp {
        op: BinOp,
        lhs: ExprId,
        rhs: ExprId,
    },
    UniOp {
        op: UniOp,
        expr: ExprId,
    },
    Index {
        base: ExprId,
        index: ExprId,
    },
    Call {
        callee: ExprId,
        args: ArgListId,
    },

    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
