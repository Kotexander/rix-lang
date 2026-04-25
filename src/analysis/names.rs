use crate::{
    ast::{AstView, idents::IdentId},
    errors::Errors,
    tir::def::DefId,
};

pub struct NameResolutions {
    map: Vec<Option<DefId>>,
}
impl NameResolutions {
    pub fn new(len: usize) -> Self {
        let map = vec![None; len];
        Self { map }
    }
    pub fn get(&self, ident_id: IdentId) -> Option<DefId> {
        self.map[ident_id.get() as usize]
    }
}

struct Resolver<'a> {
    resolutions: NameResolutions,
    view: AstView<'a>,
    errors: &'a mut Errors,
}
