use crate::{ast::AstView, errors::Errors, tir};

mod names;

const ATOM_MAP: &[(&str, tir::typ::AtomType)] = &[
    ("u8", tir::typ::AtomType::U8),
    ("u16", tir::typ::AtomType::U16),
    ("u32", tir::typ::AtomType::U32),
    ("u64", tir::typ::AtomType::U64),
    ("uptr", tir::typ::AtomType::UPtr),
    ("i8", tir::typ::AtomType::I8),
    ("i16", tir::typ::AtomType::I16),
    ("i32", tir::typ::AtomType::I32),
    ("i64", tir::typ::AtomType::I64),
    ("iptr", tir::typ::AtomType::IPtr),
    ("bool", tir::typ::AtomType::Bool),
    ("void", tir::typ::AtomType::Void),
];

pub fn analyse(view: AstView, errors: &mut Errors) -> names::NameResolutions {
    let mut defs = tir::def::Defs::new();
    let mut typs = tir::typ::Typs::new();

    let names = names::resolve_names(view, &mut defs, &mut typs, errors);

    names
}
