use super::{UniqueSymbol, expr, typ};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stmt {
    Expr(expr::ExprId),
    VarDecl {
        name: UniqueSymbol,
        value: expr::ExprId,
        typ: Option<typ::TypeId>,
    },
    Return(Option<expr::ExprId>),
}
