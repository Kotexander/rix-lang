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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmtKind {
    Expr(expr::ExprId),
    VarDecl {
        ident: IdentId,
        expr: expr::ExprId,
        typ: Option<typ::TypeId>,
    },
    Return(Option<expr::ExprId>),
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
