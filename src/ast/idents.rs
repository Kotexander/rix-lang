use super::arena::{Arena, ArenaId};
use crate::lexer;
use crate::strings::StrId;

/// An identifier with its original span information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ident {
    pub str: StrId,
    pub span: lexer::Span,
}
impl Ident {
    pub fn new(str: StrId, span: lexer::Span) -> Self {
        Self { str, span }
    }
}

pub type IdentArena = Arena<Ident>;
pub type IdentId = ArenaId<Ident>;

pub struct IdentView<'a> {
    view: super::AstView<'a>,
    id: IdentId,
}
impl<'a> IdentView<'a> {
    pub fn new(view: super::AstView<'a>, id: IdentId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> IdentId {
        self.id
    }
    pub fn str_id(&self) -> StrId {
        self.view.ast.idents[self.id].str
    }
    pub fn span(&self) -> lexer::Span {
        self.view.ast.idents[self.id].span
    }
    pub fn str(&self) -> &'a str {
        self.view.interner.resolve(self.str_id())
    }
}
