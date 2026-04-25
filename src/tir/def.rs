use std::num::NonZeroU32;

use super::typ;
use crate::{lexer, strings::StrId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DefId(NonZeroU32);
impl DefId {
    fn new(id: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(id + 1) })
    }
    fn get(self) -> u32 {
        self.0.get() - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Def {
    pub str: StrId,
    pub span: lexer::Span,
    pub typ: typ::TypeId,
}

#[derive(Debug, Clone)]
pub struct Defs {
    defs: Vec<Def>,
}
impl Defs {
    pub fn new() -> Self {
        Self { defs: Vec::new() }
    }
    pub fn insert(&mut self, str: StrId, span: lexer::Span, typ: typ::TypeId) -> DefId {
        let id = DefId::new(self.defs.len() as u32);
        self.defs.push(Def { typ, str, span });
        id
    }
}
impl std::ops::Index<DefId> for Defs {
    type Output = Def;
    fn index(&self, id: DefId) -> &Self::Output {
        &self.defs[id.get() as usize]
    }
}
