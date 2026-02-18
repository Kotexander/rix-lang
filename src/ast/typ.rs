use super::arena::{Arena, ArenaId};
use super::symbols::SymbolId;
use crate::lexer;

pub type TypeArena = Arena<Type>;
pub type TypeId = ArenaId<Type>;

impl TypeArena {
    pub fn add(&mut self, kind: TypeKind, span: lexer::Span) -> TypeId {
        self.alloc(Type::new(kind, span))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Identifier(SymbolId),
    Ptr(TypeId),
    Error,
}

#[derive(Debug, Clone, Copy)]
pub struct Type {
    pub kind: TypeKind,
    pub span: lexer::Span,
}
impl Type {
    pub fn new(kind: TypeKind, span: lexer::Span) -> Self {
        Self { kind, span }
    }
}
