use super::UniqueSymbol;
use crate::lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Identifier(UniqueSymbol),
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
