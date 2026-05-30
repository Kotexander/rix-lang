use crate::{
    ast::idents::IdentId,
    tir::{def::DefId, typ::TypId},
};

#[derive(Debug, Clone, Copy)]
enum Kind {
    Def(DefId),
    Typ(TypId),
    None,
}

pub struct NameResolutions {
    ident_map: Vec<Kind>,
}
impl NameResolutions {
    pub fn new(num_idents: usize) -> Self {
        let ident_map = vec![Kind::None; num_idents];
        Self { ident_map }
    }
    pub fn set_def(&mut self, id: IdentId, def_id: DefId) {
        self.ident_map[id.idx()] = Kind::Def(def_id);
    }
    pub fn set_typ(&mut self, id: IdentId, typ_id: TypId) {
        self.ident_map[id.idx()] = Kind::Typ(typ_id);
    }
    pub fn get_def(&self, id: IdentId) -> Option<DefId> {
        match self.ident_map[id.idx()] {
            Kind::Def(def_id) => Some(def_id),
            _ => None,
        }
    }
    pub fn get_typ(&self, id: IdentId) -> Option<TypId> {
        match self.ident_map[id.idx()] {
            Kind::Typ(typ_id) => Some(typ_id),
            _ => None,
        }
    }
}
