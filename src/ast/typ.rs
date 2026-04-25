use super::idents::{IdentId, IdentView};
use crate::{arena::ArenaId, define_view, lexer};

pub type TypeId = ArenaId<Type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// T
    Identifier(IdentId),
    /// *T
    Ptr(TypeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: lexer::Span,
}
impl Type {
    pub fn new(kind: TypeKind, span: lexer::Span) -> Self {
        Self { kind, span }
    }
}

/// See [`TypeKind`]
#[derive(Debug, Clone, Copy)]
pub enum TypeKindView<'a> {
    Identifier(IdentView<'a>),
    Ptr(TypeView<'a>),
}

define_view!(TypeView, Type, TypeId, types);
impl<'a> TypeView<'a> {
    pub fn kind(&self) -> TypeKindView<'a> {
        match &self.node().kind {
            TypeKind::Identifier(id) => TypeKindView::Identifier(IdentView::new(self.view, *id)),
            TypeKind::Ptr(id) => TypeKindView::Ptr(Self::new(self.view, *id)),
        }
    }
}
