use super::arena::{Arena, ArenaId};
use super::symbols::SymbolId;
use super::{expr, typ};

pub type StmtArena = Arena<Stmt>;
pub type StmtId = ArenaId<Stmt>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stmt {
    Expr(expr::ExprId),
    VarDecl {
        name: SymbolId,
        value: expr::ExprId,
        typ: Option<typ::TypeId>,
    },
    Return(Option<expr::ExprId>),
}
