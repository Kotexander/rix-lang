use super::{UniqueSymbol, stmt, typ};
use crate::lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub(super) u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Function {
        name: UniqueSymbol,
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
    pub name: UniqueSymbol,
}
impl Arg {
    pub fn new(kind: ArgKind, name: UniqueSymbol) -> Self {
        Self { kind, name }
    }
}
