use super::arena::{Arena, ArenaId};
use super::idents::IdentId;
use super::{stmt, typ};
use crate::lexer;

pub type ItemArena = Arena<Item>;
pub type ItemId = ArenaId<Item>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Fun {
        ident: IdentId,
        params: Vec<Param>,
        rett: Option<typ::TypeId>,
        body: Option<Vec<stmt::StmtId>>,
    },
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
