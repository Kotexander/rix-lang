use crate::{arena::ArenaId, define_view, lexer, strings::StrId};

pub type StrLitId = ArenaId<StrLit>;
pub type NumLitId = ArenaId<NumLit>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrLit {
    /// Parsed string literal, without quotes and escape sequences.
    pub str: StrId,
    pub span: lexer::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumLit {
    pub num: u128,
    pub span: lexer::Span,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct FloatLit {
//     pub float: f64,
//     pub span: lexer::Span,
// }

define_view!(StrLitView, StrLit, StrLitId, strings);
impl<'a> StrLitView<'a> {
    pub fn str_id(&self) -> StrId {
        self.node().str
    }
    /// returns the resolved string literal, without quotes and escape sequences.
    pub fn str(&self) -> &'a str {
        self.view.interner.resolve(self.str_id())
    }
    /// returns the original string literal, with quotes and escape sequences.
    pub fn src(&self) -> &'a str {
        &self.view.src[self.span()]
    }
}

define_view!(NumLitView, NumLit, NumLitId, numbers);
impl<'a> NumLitView<'a> {
    pub fn num(&self) -> u128 {
        self.node().num
    }
    pub fn str(&self) -> &'a str {
        &self.view.src[self.span()]
    }
}
