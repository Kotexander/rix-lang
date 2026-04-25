use super::{
    expr::{ExprId, ExprView},
    idents::{IdentId, IdentView},
    typ::{TypeId, TypeView},
};
use crate::{
    arena::{ArenaId, ArenaRange},
    define_view, lexer,
};

pub type StmtId = ArenaId<Stmt>;
pub type CondBlockId = ArenaId<CondBlock>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmtKind {
    Expr(ExprId),
    VarDecl {
        ident: IdentId,
        expr: ExprId,
        typ: Option<TypeId>,
    },
    Assign {
        lhs: ExprId,
        rhs: ExprId,
    },
    Return(Option<ExprId>),
    If {
        /// [0] is the main `if` block, [1..] are the `else if` blocks
        elifs: ArenaRange<CondBlockId>,
        els: Option<ArenaRange<StmtId>>,
    },
    While(CondBlockId),
    Break,
    Continue,
}

/// Conditional block, used for if and while statements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CondBlock {
    pub cond: ExprId,
    pub block: ArenaRange<StmtId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: lexer::Span,
}
impl Stmt {
    pub fn new(kind: StmtKind, span: lexer::Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StmtListView<'a> {
    view: super::AstView<'a>,
    ids: ArenaRange<StmtId>,
}
impl<'a> StmtListView<'a> {
    pub fn new(view: super::AstView<'a>, ids: ArenaRange<StmtId>) -> Self {
        Self { ids, view }
    }
    pub fn iter(&self) -> impl Iterator<Item = StmtView<'a>> {
        self.view.ast.stmts_lists[self.ids]
            .iter()
            .map(|id| StmtView::new(self.view, *id))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CondBlockView<'a> {
    view: super::AstView<'a>,
    block: CondBlockId,
}
impl<'a> CondBlockView<'a> {
    pub fn new(view: super::AstView<'a>, block: CondBlockId) -> Self {
        Self { block, view }
    }
    pub fn id(&self) -> CondBlockId {
        self.block
    }
    pub fn cond(&self) -> ExprView<'a> {
        ExprView::new(self.view, self.view.ast.cond_blocks[self.block].cond)
    }
    pub fn block(&self) -> StmtListView<'a> {
        StmtListView::new(self.view, self.view.ast.cond_blocks[self.block].block)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CondBlockListView<'a> {
    view: super::AstView<'a>,
    ids: ArenaRange<CondBlockId>,
}
impl<'a> CondBlockListView<'a> {
    pub fn new(view: super::AstView<'a>, ids: ArenaRange<CondBlockId>) -> Self {
        Self { ids, view }
    }
    pub fn iter(&self) -> impl Iterator<Item = CondBlockView<'a>> {
        self.view.ast.cond_blocks_lists[self.ids]
            .iter()
            .map(|id| CondBlockView::new(self.view, *id))
    }
}

/// See [`StmtKind`]
#[derive(Debug, Clone, Copy)]
pub enum StmtKindView<'a> {
    Expr(ExprView<'a>),
    VarDecl {
        ident: IdentView<'a>,
        expr: ExprView<'a>,
        typ: Option<TypeView<'a>>,
    },
    Assign {
        lhs: ExprView<'a>,
        rhs: ExprView<'a>,
    },
    Return(Option<ExprView<'a>>),
    If {
        elifs: CondBlockListView<'a>,
        els: Option<StmtListView<'a>>,
    },
    While(CondBlockView<'a>),
    Break,
    Continue,
}

define_view!(StmtView, Stmt, StmtId, stmts);
impl<'a> StmtView<'a> {
    pub fn kind(&self) -> StmtKindView<'a> {
        match &self.node().kind {
            StmtKind::Expr(expr) => StmtKindView::Expr(ExprView::new(self.view, *expr)),
            StmtKind::VarDecl { ident, expr, typ } => StmtKindView::VarDecl {
                ident: IdentView::new(self.view, *ident),
                expr: ExprView::new(self.view, *expr),
                typ: typ.map(|typ| TypeView::new(self.view, typ)),
            },
            StmtKind::Assign { lhs, rhs } => StmtKindView::Assign {
                lhs: ExprView::new(self.view, *lhs),
                rhs: ExprView::new(self.view, *rhs),
            },
            StmtKind::Return(arena_id) => {
                StmtKindView::Return(arena_id.map(|id| ExprView::new(self.view, id)))
            }
            StmtKind::If { elifs, els } => StmtKindView::If {
                elifs: CondBlockListView::new(self.view, *elifs),
                els: els.map(|stmts| StmtListView::new(self.view, stmts)),
            },
            StmtKind::While(cond_block) => {
                StmtKindView::While(CondBlockView::new(self.view, *cond_block))
            }
            StmtKind::Break => StmtKindView::Break,
            StmtKind::Continue => StmtKindView::Continue,
        }
    }
}
