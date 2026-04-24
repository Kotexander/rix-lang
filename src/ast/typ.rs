use super::arena::{Arena, ArenaId};
use super::idents::{IdentId, IdentView};
use crate::lexer;

pub type TypeArena = Arena<Type>;
pub type TypeId = ArenaId<Type>;

impl TypeArena {
    pub fn add(&mut self, kind: TypeKind, span: lexer::Span) -> TypeId {
        self.alloc(Type::new(kind, span))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind<T = TypeId, I = IdentId> {
    Identifier(I),
    Ptr(T),
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

pub struct TypeView<'a> {
    view: super::AstView<'a>,
    id: TypeId,
}
impl<'a> TypeView<'a> {
    pub fn new(view: super::AstView<'a>, id: TypeId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> TypeId {
        self.id
    }
    pub fn span(&self) -> lexer::Span {
        self.view.ast.types[self.id].span
    }
    pub fn kind(&self) -> TypeKind<TypeView<'a>, IdentView<'a>> {
        let typ = &self.view.ast.types[self.id];
        match &typ.kind {
            TypeKind::Identifier(id) => TypeKind::Identifier(IdentView::new(self.view, *id)),
            TypeKind::Ptr(id) => TypeKind::Ptr(Self::new(self.view, *id)),
        }
    }
}
