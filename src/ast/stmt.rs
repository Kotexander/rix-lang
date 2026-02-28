use super::arena::{Arena, ArenaId};
use super::idents::IdentId;
use super::{expr, typ};

pub type StmtArena = Arena<Stmt>;
pub type StmtId = ArenaId<Stmt>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stmt {
    Expr(expr::ExprId),
    VarDecl {
        ident: IdentId,
        expr: expr::ExprId,
        typ: Option<typ::TypeId>,
    },
    Return(Option<expr::ExprId>),
}
