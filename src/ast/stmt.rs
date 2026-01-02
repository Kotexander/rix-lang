#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(pub(super) u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Expr(super::expr::ExprId),
    VarDecl {
        name: super::Symbol,
        name_span: crate::lexer::Span,
        value: super::expr::ExprId,
    },
}
