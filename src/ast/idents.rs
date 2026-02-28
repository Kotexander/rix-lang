use crate::lexer;
use std::{collections::HashMap, sync::Arc};

/// An identifier with its original span information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ident<S> {
    pub ident: S,
    pub span: lexer::Span,
}
impl<S> Ident<S> {
    pub fn new(ident: S, span: lexer::Span) -> Self {
        Self { ident, span }
    }
    pub fn map<T>(self, map: impl FnOnce(S) -> T) -> Ident<T> {
        Ident {
            ident: map(self.ident),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentId(u32);

#[derive(Debug, Default)]
pub struct Idents {
    idents: Vec<Ident<StringId>>,
    interner: StringInterner,
}
impl Idents {
    pub fn add(&mut self, ident: &str, span: lexer::Span) -> IdentId {
        let symbol = Ident::new(ident, span);
        let id = self.idents.len() as u32;
        self.idents.push(symbol.map(|s| self.interner.intern(s)));
        IdentId(id)
    }
    pub fn str(&self, id: IdentId) -> &str {
        self.interner.resolve(self.idents[id.0 as usize].ident)
    }
    pub fn string_id(&self, id: IdentId) -> StringId {
        self.idents[id.0 as usize].ident
    }
    pub fn span(&self, id: IdentId) -> lexer::Span {
        self.idents[id.0 as usize].span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringId(u32);

#[derive(Debug, Default)]
struct StringInterner {
    strings: Vec<Arc<str>>,
    map: HashMap<Arc<str>, StringId>,
}
impl StringInterner {
    pub fn intern(&mut self, string: &str) -> StringId {
        if let Some(id) = self.map.get(string) {
            *id
        } else {
            let id = self.strings.len() as u32;
            let arc_str = Arc::<str>::from(string);
            self.strings.push(arc_str.clone());
            let string_id = StringId(id);
            self.map.insert(arc_str, string_id);
            string_id
        }
    }

    pub fn resolve(&self, string: StringId) -> &str {
        self.strings[string.0 as usize].as_ref()
    }
}
