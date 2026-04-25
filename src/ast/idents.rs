use crate::{arena::ArenaId, define_view, lexer, strings::StrId};

pub type IdentId = ArenaId<Ident>;

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

define_view!(IdentView, Ident, IdentId, idents);
impl<'a> IdentView<'a> {
    pub fn str_id(&self) -> StrId {
        self.node().str
    }
    pub fn str(&self) -> &'a str {
        self.view.interner.resolve(self.str_id())
    }
}
