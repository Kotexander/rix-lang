use super::{
    arena::{Arena, ArenaId},
    expr::{ExprId, ExprView},
    idents::{IdentId, IdentView},
    typ::{TypeId, TypeView},
};
use crate::lexer;

pub type StmtArena = Arena<Stmt>;
pub type StmtId = ArenaId<Stmt>;

impl StmtArena {
    pub fn add(&mut self, kind: StmtKind, span: lexer::Span) -> StmtId {
        self.alloc(Stmt::new(kind, span))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind<S = StmtId, T = TypeId, I = IdentId, E = ExprId> {
    Expr(E),
    VarDecl {
        ident: I,
        expr: E,
        typ: Option<T>,
    },
    Assign {
        lhs: E,
        rhs: E,
    },
    Return(Option<E>),
    If {
        /// first [CondBlock] is the main `if` block, the rest are `else if` blocks
        elifs: Vec<CondBlock<S, E>>,
        els: Option<Vec<S>>,
    },
    While(CondBlock<S, E>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Conditional block, used for if and while statements
pub struct CondBlock<S = StmtId, E = ExprId> {
    pub cond: E,
    pub body: Vec<S>,
}
pub type CondBlockView<'a> = CondBlock<StmtView<'a>, ExprView<'a>>;

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

pub struct StmtView<'a> {
    view: super::AstView<'a>,
    id: StmtId,
}
impl<'a> StmtView<'a> {
    pub fn new(view: super::AstView<'a>, id: StmtId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> StmtId {
        self.id
    }
    pub fn span(&self) -> lexer::Span {
        self.view.ast.stmts[self.id].span
    }
    pub fn kind(&self) -> StmtKind<StmtView<'a>, TypeView<'a>, IdentView<'a>, ExprView<'a>> {
        let stmt = &self.view.ast.stmts[self.id];
        match &stmt.kind {
            StmtKind::Expr(expr) => StmtKind::Expr(ExprView::new(self.view, *expr)),
            StmtKind::VarDecl { ident, expr, typ } => StmtKind::VarDecl {
                ident: IdentView::new(self.view, *ident),
                expr: ExprView::new(self.view, *expr),
                typ: typ.map(|typ| TypeView::new(self.view, typ)),
            },
            StmtKind::Assign { lhs, rhs } => StmtKind::Assign {
                lhs: ExprView::new(self.view, *lhs),
                rhs: ExprView::new(self.view, *rhs),
            },
            StmtKind::Return(expr) => {
                StmtKind::Return(expr.map(|expr| ExprView::new(self.view, expr)))
            }
            StmtKind::If { elifs, els } => StmtKind::If {
                elifs: elifs
                    .iter()
                    .map(|cond_block| CondBlock {
                        cond: ExprView::new(self.view, cond_block.cond),
                        body: cond_block
                            .body
                            .iter()
                            .map(|stmt| StmtView::new(self.view, *stmt))
                            .collect(),
                    })
                    .collect(),
                els: els.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|stmt| StmtView::new(self.view, *stmt))
                        .collect()
                }),
            },
            StmtKind::While(cond_block) => StmtKind::While(CondBlock {
                cond: ExprView::new(self.view, cond_block.cond),
                body: cond_block
                    .body
                    .iter()
                    .map(|stmt| StmtView::new(self.view, *stmt))
                    .collect(),
            }),
            StmtKind::Break => StmtKind::Break,
            StmtKind::Continue => StmtKind::Continue,
        }
    }
}
