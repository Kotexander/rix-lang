use crate::ast::idents::IdentId;

use super::typ;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Def {
    pub ident: IdentId,
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
    pub fn insert(&mut self, ident: IdentId, typ: typ::TypeId) -> DefId {
        let id = DefId(self.defs.len() as u32);
        self.defs.push(Def { ident, typ });
        id
    }
}
impl std::ops::Index<DefId> for Defs {
    type Output = Def;
    fn index(&self, id: DefId) -> &Self::Output {
        &self.defs[id.0 as usize]
    }
}
