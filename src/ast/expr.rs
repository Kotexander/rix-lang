use super::{
    idents::{IdentId, IdentView},
    literals::{NumLitId, NumLitView, StrLitId, StrLitView},
};
use crate::{
    arena::{ArenaId, ArenaRange},
    define_view, lexer,
};

pub type ExprId = ArenaId<Expr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprKind {
    Identifier(IdentId),
    Number(NumLitId),
    String(StrLitId),

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
        args: ArenaRange<ExprId>,
    },
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

#[derive(Debug, Clone, Copy)]
pub struct ExprViewList<'a> {
    view: super::AstView<'a>,
    ids: ArenaRange<ExprId>,
}
impl<'a> ExprViewList<'a> {
    pub fn new(view: super::AstView<'a>, ids: ArenaRange<ExprId>) -> Self {
        Self { ids, view }
    }
    pub fn iter(&self) -> impl Iterator<Item = ExprView<'a>> {
        self.view.ast.exprs_lists[self.ids]
            .iter()
            .map(|id| ExprView::new(self.view, *id))
    }
}

/// See [`ExprKind`]
#[derive(Debug, Clone, Copy)]
pub enum ExprKindView<'a> {
    Identifier(IdentView<'a>),
    Number(NumLitView<'a>),
    String(StrLitView<'a>),

    Group(ExprView<'a>),
    BinOp {
        op: BinOp,
        lhs: ExprView<'a>,
        rhs: ExprView<'a>,
    },
    UniOp {
        op: UniOp,
        expr: ExprView<'a>,
    },
    Index {
        base: ExprView<'a>,
        index: ExprView<'a>,
    },
    Call {
        callee: ExprView<'a>,
        args: ExprViewList<'a>,
    },
}

define_view!(ExprView, Expr, ExprId, exprs);
impl<'a> ExprView<'a> {
    pub fn kind(&self) -> ExprKindView<'a> {
        match &self.node().kind {
            ExprKind::Identifier(id) => ExprKindView::Identifier(IdentView::new(self.view, *id)),
            ExprKind::Number(id) => ExprKindView::Number(NumLitView::new(self.view, *id)),
            ExprKind::String(id) => ExprKindView::String(StrLitView::new(self.view, *id)),
            ExprKind::Group(id) => ExprKindView::Group(Self::new(self.view, *id)),
            ExprKind::BinOp { op, lhs, rhs } => ExprKindView::BinOp {
                op: *op,
                lhs: Self::new(self.view, *lhs),
                rhs: Self::new(self.view, *rhs),
            },
            ExprKind::UniOp { op, expr } => ExprKindView::UniOp {
                op: *op,
                expr: Self::new(self.view, *expr),
            },
            ExprKind::Index { base, index } => ExprKindView::Index {
                base: Self::new(self.view, *base),
                index: Self::new(self.view, *index),
            },
            ExprKind::Call { callee, args } => ExprKindView::Call {
                callee: Self::new(self.view, *callee),
                args: ExprViewList::new(self.view, *args),
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
