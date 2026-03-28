use crate::{
    ast::{expr::ExprId, idents::IdentId},
    tir,
};
use std::collections::HashMap;

pub struct Resolutions {
    pub expr_types: HashMap<ExprId, tir::typ::TypeId>,
    pub def_resolutions: HashMap<IdentId, tir::def::DefId>,
}
impl Resolutions {
    pub fn new() -> Self {
        Self {
            expr_types: HashMap::new(),
            def_resolutions: HashMap::new(),
        }
    }
}
