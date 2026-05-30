use crate::{
    arena::{Arena, ArenaId},
    ast::idents::IdentId,
};

pub type DefId = ArenaId<Def>;
pub type Defs = Arena<Def>;

#[derive(Debug, Clone, Copy)]
pub struct Def {
    pub ident: IdentId,
}
