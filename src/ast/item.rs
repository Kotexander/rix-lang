use super::{
    idents::{IdentId, IdentView},
    stmt::{StmtId, StmtListView},
    typ::{TypeId, TypeView},
};
use crate::{
    arena::{ArenaId, ArenaRange},
    lexer,
};

pub type ItemId = ArenaId<Item>;
pub type ParamId = ArenaId<Param>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Fun(Fun),
}
impl Item {
    pub fn as_fun(&self) -> Option<&Fun> {
        match self {
            Item::Fun(fun) => Some(fun),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fun {
    pub ident: IdentId,
    pub params: ArenaRange<Param>,
    pub ret_type: Option<TypeId>,
    pub body: Option<ArenaRange<StmtId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    Type(TypeId),
    Variadic(lexer::Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    pub typ: ParamType,
    pub ident: IdentId,
}
impl Param {
    pub fn new(typ: ParamType, ident: IdentId) -> Self {
        Self { typ, ident }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParamTypeView<'a> {
    Type(TypeView<'a>),
    Variadic(lexer::Span),
}
#[derive(Debug, Clone, Copy)]
pub struct ParamView<'a> {
    view: super::AstView<'a>,
    id: ParamId,
}
impl<'a> ParamView<'a> {
    pub fn new(view: super::AstView<'a>, id: ParamId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> ParamId {
        self.id
    }
    pub fn typ(&self) -> ParamTypeView<'a> {
        match &self.node().typ {
            ParamType::Type(id) => ParamTypeView::Type(TypeView::new(self.view, *id)),
            ParamType::Variadic(span) => ParamTypeView::Variadic(*span),
        }
    }
    pub fn ident(&self) -> IdentView<'a> {
        IdentView::new(self.view, self.node().ident)
    }

    pub fn node(&self) -> &'a Param {
        &self.view.ast.params[self.id]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FunView<'a> {
    view: super::AstView<'a>,
    fun: &'a Fun,
}
impl<'a> FunView<'a> {
    pub fn new(view: super::AstView<'a>, fun: &'a Fun) -> Self {
        Self { view, fun }
    }
    pub fn ident(&self) -> IdentView<'a> {
        IdentView::new(self.view, self.fun.ident)
    }
    pub fn params(&self) -> impl Iterator<Item = ParamView<'a>> {
        self.fun
            .params
            .iter()
            .map(|id| ParamView::new(self.view, id))
    }
    pub fn ret_type(&self) -> Option<TypeView<'a>> {
        self.fun.ret_type.map(|id| TypeView::new(self.view, id))
    }
    pub fn body(&self) -> Option<StmtListView<'a>> {
        self.fun.body.map(|ids| StmtListView::new(self.view, ids))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ItemKindView<'a> {
    Fun(FunView<'a>),
}
#[derive(Debug, Clone, Copy)]
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
    pub fn kind(&self) -> ItemKindView<'a> {
        match &self.node() {
            Item::Fun(fun) => ItemKindView::Fun(FunView::new(self.view, fun)),
        }
    }

    pub fn node(&self) -> &'a Item {
        &self.view.ast.items[self.id]
    }
}
