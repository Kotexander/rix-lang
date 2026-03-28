use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// NOTE: Remember to update [Self::ALL] when changing this enum.
pub enum AtomType {
    Void,
    Bool,
    U8,
    U16,
    U32,
    U64,
    UPtr,
    I8,
    I16,
    I32,
    I64,
    IPtr,
}
impl AtomType {
    // NOTE: The order of this array must match the order of the enum.
    const ALL: &[AtomType] = &[
        AtomType::Void,
        AtomType::Bool,
        AtomType::U8,
        AtomType::U16,
        AtomType::U32,
        AtomType::U64,
        AtomType::UPtr,
        AtomType::I8,
        AtomType::I16,
        AtomType::I32,
        AtomType::I64,
        AtomType::IPtr,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunType {
    pub params: Vec<TypeId>,
    pub ret_type: TypeId,
    pub varargs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Atom(AtomType),
    Ptr(TypeId),
    Fun(FunType),
    Error,
}
impl Type {
    pub fn as_fun(&self) -> Option<&FunType> {
        match self {
            Type::Fun(func_type) => Some(func_type),
            _ => None,
        }
    }
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

pub struct Typs {
    types: Vec<Arc<Type>>,
    map: HashMap<Arc<Type>, TypeId>,
}
impl Typs {
    pub fn new() -> Self {
        let mut slf = Self {
            types: Vec::new(),
            map: HashMap::new(),
        };

        slf.insert(Type::Error);
        for atom in AtomType::ALL {
            slf.insert(Type::Atom(*atom));
        }

        slf
    }
    pub fn atom(&self, atom: AtomType) -> TypeId {
        TypeId(atom as u32 + 1)
    }
    pub fn error(&self) -> TypeId {
        TypeId(0)
    }
    pub fn ptr(&mut self, inner: TypeId) -> TypeId {
        self.insert(Type::Ptr(inner))
    }
    pub fn fun(&mut self, fun: FunType) -> TypeId {
        self.insert(Type::Fun(fun))
    }

    fn insert(&mut self, typ: Type) -> TypeId {
        if let Some(id) = self.map.get(&typ) {
            *id
        } else {
            let typ = Arc::new(typ);
            let id = TypeId(self.types.len() as u32);
            self.types.push(typ.clone());
            self.map.insert(typ, id);
            id
        }
    }
}
impl std::ops::Index<TypeId> for Typs {
    type Output = Type;
    fn index(&self, id: TypeId) -> &Self::Output {
        &self.types[id.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_typ_ids() {
        let mut typs = Typs::new();

        // make sure that error() and insert(Type::Error) return the same id
        assert_eq!(typs.error(), typs.insert(Type::Error));

        // make sure that atom() and insert(Type::Atom) return the same id
        for atom in AtomType::ALL {
            assert_eq!(typs.atom(*atom), typs.insert(Type::Atom(*atom)));
        }
    }
}
