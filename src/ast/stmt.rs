use super::arena::{Arena, ArenaId};
use super::idents::IdentId;
use super::{expr, typ};
use crate::lexer;

pub type StmtArena = Arena<Stmt>;
pub type StmtId = ArenaId<Stmt>;

impl StmtArena {
    pub fn add(&mut self, kind: StmtKind, span: lexer::Span) -> StmtId {
        self.alloc(Stmt::new(kind, span))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Expr(expr::ExprId),
    VarDecl {
        ident: IdentId,
        expr: expr::ExprId,
        typ: Option<typ::TypeId>,
    },
    Assign {
        lhs: expr::ExprId,
        rhs: expr::ExprId,
    },
    Return(Option<expr::ExprId>),
    If {
        /// first elif block is the main if block, the rest are else if blocks
        elifs: Vec<CondBlock>,
        els: Option<Vec<StmtId>>,
    },
    While(CondBlock),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Conditional block, used for if and while statements
pub struct CondBlock {
    pub cond: expr::ExprId,
    pub body: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: lexer::Span,
}
impl Stmt {
    pub fn new(kind: StmtKind, span: lexer::Span) -> Self {
        Self { kind, span }
    }
}
