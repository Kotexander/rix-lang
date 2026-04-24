use super::{
    arena::{Arena, ArenaId},
    idents::{IdentId, IdentView},
    stmt::{StmtId, StmtView},
    typ::{TypeId, TypeView},
};
use crate::lexer;

pub type ItemArena = Arena<Item>;
pub type ItemId = ArenaId<Item>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<T = TypeId, S = StmtId, I = IdentId> {
    Fun(Fun<T, S, I>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fun<T = TypeId, S = StmtId, I = IdentId> {
    pub ident: I,
    pub params: Vec<Param<T, I>>,
    pub ret_type: Option<T>,
    pub body: Option<Vec<S>>,
}
pub type FunView<'a> = Fun<TypeView<'a>, StmtView<'a>, IdentView<'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType<T = TypeId> {
    Type(T),
    Variadic(lexer::Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param<T = TypeId, I = IdentId> {
    pub typ: ParamType<T>,
    pub ident: I,
}
impl Param {
    pub fn new(typ: ParamType, ident: IdentId) -> Self {
        Self { typ, ident }
    }
}

pub struct ItemView<'a> {
    view: super::AstView<'a>,
    id: ItemId,
}
impl<'a> ItemView<'a> {
    pub fn new(view: super::AstView<'a>, id: ItemId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> ItemId {
        self.id
    }
    /// TODO: don't allocate
    pub fn kind(&self) -> Item<TypeView<'a>, StmtView<'a>, IdentView<'a>> {
        let item = &self.view.ast.items[self.id];
        match &item {
            Item::Fun(fun) => Item::Fun(FunView {
                ident: IdentView::new(self.view, fun.ident),
                params: fun
                    .params
                    .iter()
                    .map(|param| Param {
                        typ: match param.typ {
                            ParamType::Type(typ) => ParamType::Type(TypeView::new(self.view, typ)),
                            ParamType::Variadic(span) => ParamType::Variadic(span),
                        },
                        ident: IdentView::new(self.view, param.ident),
                    })
                    .collect(),
                ret_type: fun.ret_type.map(|typ| TypeView::new(self.view, typ)),
                body: fun.body.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|stmt| StmtView::new(self.view, *stmt))
                        .collect()
                }),
            }),
        }
    }
}
