use super::arena::{Arena, ArenaId};
use crate::{lexer, strings::StrId};

pub type StrLitArena = Arena<StrLit>;
pub type StrLitId = ArenaId<StrLit>;

pub type NumLitArena = Arena<NumLit>;
pub type NumLitId = ArenaId<NumLit>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrLit {
    /// Parsed string literal, without quotes and escape sequences.
    pub str: StrId,
    /// The original source of the string literal, including quotes and escape sequences.
    pub src: StrId,
    pub span: lexer::Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumLit {
    pub num: u128,
    pub src: StrId,
    pub span: lexer::Span,
}

pub struct StrLitView<'a> {
    view: super::AstView<'a>,
    id: StrLitId,
}
impl<'a> StrLitView<'a> {
    pub fn new(view: super::AstView<'a>, id: StrLitId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> StrLitId {
        self.id
    }
    pub fn str_id(&self) -> StrId {
        self.view.ast.strings[self.id].str
    }
    pub fn src_id(&self) -> StrId {
        self.view.ast.strings[self.id].src
    }
    pub fn str(&self) -> &'a str {
        self.view.interner.resolve(self.str_id())
    }
    pub fn src(&self) -> &'a str {
        self.view.interner.resolve(self.src_id())
    }
    pub fn span(&self) -> lexer::Span {
        self.view.ast.strings[self.id].span
    }
}

pub struct NumLitView<'a> {
    view: super::AstView<'a>,
    id: NumLitId,
}
impl<'a> NumLitView<'a> {
    pub fn new(view: super::AstView<'a>, id: NumLitId) -> Self {
        Self { id, view }
    }
    pub fn id(&self) -> NumLitId {
        self.id
    }
    pub fn num(&self) -> u128 {
        self.view.ast.numbers[self.id].num
    }
    pub fn src_id(&self) -> StrId {
        self.view.ast.numbers[self.id].src
    }
    pub fn src(&self) -> &'a str {
        self.view.interner.resolve(self.src_id())
    }
    pub fn span(&self) -> lexer::Span {
        self.view.ast.numbers[self.id].span
    }
}
