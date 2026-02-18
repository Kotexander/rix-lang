use super::arena::{Arena, ArenaId};
use super::symbols::SymbolId;
use super::{stmt, typ};
use crate::lexer;

pub type ItemArena = Arena<Item>;
pub type ItemId = ArenaId<Item>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Function {
        name: SymbolId,
        args: Vec<Arg>,
        rett: Option<typ::TypeId>,
        body: Option<Vec<stmt::StmtId>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    Type(typ::TypeId),
    Variadic(lexer::Span),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arg {
    pub kind: ArgKind,
    pub name: SymbolId,
}
impl Arg {
    pub fn new(kind: ArgKind, name: SymbolId) -> Self {
        Self { kind, name }
    }
}
