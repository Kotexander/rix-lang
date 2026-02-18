use crate::lexer;
use std::{collections::HashMap, sync::Arc};

/// A symbol with its original span information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol<S> {
    pub sym: S,
    pub span: lexer::Span,
}
impl<S> Symbol<S> {
    pub fn new(sym: S, span: lexer::Span) -> Self {
        Self { sym, span }
    }
    pub fn map<T>(self, map: impl FnOnce(S) -> T) -> Symbol<T> {
        Symbol {
            sym: map(self.sym),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

#[derive(Debug, Default)]
pub struct Symbols {
    symbols: Vec<Symbol<StringId>>,
    interner: Interner,
}
impl Symbols {
    pub fn add(&mut self, s: &str, span: lexer::Span) -> SymbolId {
        let symbol = Symbol::new(s, span);
        let id = self.symbols.len() as u32;
        self.symbols.push(symbol.map(|s| self.interner.intern(s)));
        SymbolId(id)
    }
    pub fn name(&self, id: SymbolId) -> &str {
        self.interner.resolve(self.symbols[id.0 as usize].sym)
    }
    pub fn span(&self, id: SymbolId) -> lexer::Span {
        self.symbols[id.0 as usize].span
    }
    // pub fn get(&self, id: SymbolId) -> Symbol<&str> {
    //     self.symbols.get(id).map(|id| self.interner.resolve(id))
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StringId(u32);
#[derive(Debug, Default)]
struct Interner {
    strings: Vec<Arc<str>>,
    map: HashMap<Arc<str>, StringId>,
}
impl Interner {
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
