use super::arena::{Arena, ArenaId};
use super::idents::{IdentId, IdentView};
use super::literals::{NumLitId, NumLitView, StrLitId, StrLitView};
use crate::lexer;

pub type ExprArena = Arena<Expr>;
pub type ExprId = ArenaId<Expr>;

impl ExprArena {
    pub fn add(&mut self, kind: ExprKind, span: lexer::Span) -> ExprId {
        self.alloc(Expr::new(kind, span))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind<I = IdentId, N = NumLitId, S = StrLitId, E = ExprId> {
    Identifier(I),
    Number(N),
    String(S),

    Group(E),
    BinOp { op: BinOp, lhs: E, rhs: E },
    UniOp { op: UniOp, expr: E },
    Index { base: E, index: E },
    Call { callee: E, args: Vec<E> },
}
pub type ExprKindView<'a> = ExprKind<IdentView<'a>, NumLitView<'a>, StrLitView<'a>, ExprView<'a>>;

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

pub struct ExprView<'a> {
    view: super::AstView<'a>,
    id: ExprId,
}
impl<'a> ExprView<'a> {
    pub fn new(view: super::AstView<'a>, id: ExprId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> ExprId {
        self.id
    }
    pub fn span(&self) -> lexer::Span {
        self.view.ast.exprs[self.id].span
    }
    pub fn kind(&self) -> ExprKindView<'a> {
        let expr = &self.view.ast.exprs[self.id];
        match &expr.kind {
            ExprKind::Identifier(id) => ExprKind::Identifier(IdentView::new(self.view, *id)),
            ExprKind::Number(id) => ExprKind::Number(NumLitView::new(self.view, *id)),
            ExprKind::String(id) => ExprKind::String(StrLitView::new(self.view, *id)),
            ExprKind::Group(id) => ExprKind::Group(Self::new(self.view, *id)),
            ExprKind::BinOp { op, lhs, rhs } => ExprKind::BinOp {
                op: *op,
                lhs: Self::new(self.view, *lhs),
                rhs: Self::new(self.view, *rhs),
            },
            ExprKind::UniOp { op, expr } => ExprKind::UniOp {
                op: *op,
                expr: Self::new(self.view, *expr),
            },
            ExprKind::Index { base, index } => ExprKind::Index {
                base: Self::new(self.view, *base),
                index: Self::new(self.view, *index),
            },
            ExprKind::Call { callee, args } => ExprKind::Call {
                callee: Self::new(self.view, *callee),
                args: args.iter().map(|arg| Self::new(self.view, *arg)).collect(),
            },
        }
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
impl BinOp {
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge | BinOp::Eq | BinOp::Ne
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniOp {
    Neg,
    Not,
    Ref,
    Deref,
}
