use super::arena::{Arena, ArenaId};
use super::idents::IdentId;
use super::{stmt, typ};
use crate::lexer;

pub type ItemArena = Arena<Item>;
pub type ItemId = ArenaId<Item>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Fun(Fun),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fun {
    pub ident: IdentId,
    pub params: Vec<Param>,
    pub ret_type: Option<typ::TypeId>,
    pub body: Option<Vec<stmt::StmtId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    Type(typ::TypeId),
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
