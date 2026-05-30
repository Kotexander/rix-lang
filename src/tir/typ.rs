use std::{collections::HashMap, num::NonZeroU32, sync::Arc};

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
    pub params: Vec<TypId>,
    pub ret_type: TypId,
    pub varargs: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Atom(AtomType),
    Ptr(TypId),
    Fun(FunTypeId),
    Error,
}
impl Type {
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypId(NonZeroU32);
impl TypId {
    fn new(id: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(id + 1) })
    }
    pub fn get(self) -> u32 {
        self.0.get() - 1
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunTypeId(NonZeroU32);
impl FunTypeId {
    fn new(id: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(id + 1) })
    }
    pub fn get(self) -> u32 {
        self.0.get() - 1
    }
}

#[derive(Debug)]
pub struct Typs {
    types: Vec<Type>,
    map: HashMap<Type, TypId>,

    funs: Vec<Arc<FunType>>,
    fun_map: HashMap<Arc<FunType>, FunTypeId>,
}
impl Typs {
    pub fn new() -> Self {
        let mut slf = Self {
            types: Vec::new(),
            map: HashMap::new(),
            funs: Vec::new(),
            fun_map: HashMap::new(),
        };

        slf.insert(Type::Error);
        for atom in AtomType::ALL {
            slf.insert(Type::Atom(*atom));
        }

        slf
    }
    pub fn atom(&self, atom: AtomType) -> TypId {
        TypId::new(atom as u32 + 1)
    }
    pub fn error(&self) -> TypId {
        TypId::new(0)
    }
    pub fn ptr(&mut self, inner: TypId) -> TypId {
        self.insert(Type::Ptr(inner))
    }
    pub fn fun(&mut self, fun: FunType) -> (TypId, FunTypeId) {
        let fun = self.insert_fun(fun);
        let typ = self.insert(Type::Fun(fun));
        (typ, fun)
    }
    pub fn try_get_fun(&self, id: TypId) -> Option<&FunType> {
        match self.types.get(id.get() as usize) {
            Some(Type::Fun(fun)) => Some(&self.funs[fun.get() as usize]),
            _ => None,
        }
    }

    fn insert(&mut self, typ: Type) -> TypId {
        if let Some(id) = self.map.get(&typ) {
            *id
        } else {
            let id = TypId::new(self.types.len() as u32);
            self.types.push(typ);
            self.map.insert(typ, id);
            id
        }
    }

    fn insert_fun(&mut self, fun: FunType) -> FunTypeId {
        if let Some(id) = self.fun_map.get(&fun) {
            *id
        } else {
            let id = FunTypeId::new(self.funs.len() as u32);
            let fun = Arc::new(fun);
            self.funs.push(fun.clone());
            self.fun_map.insert(fun, id);
            id
        }
    }
}
// impl std::ops::Index<TypeId> for Typs {
//     type Output = Type;
//     fn index(&self, id: TypeId) -> &Self::Output {
//         &self.types[id.get() as usize]
//     }
// }
// impl<'a> IntoIterator for &'a Typs {
//     type Item = (TypeId, Type);
//     type IntoIter = std::iter::Map<
//         std::iter::Enumerate<std::slice::Iter<'a, Type>>,
//         fn((usize, &Type)) -> Self::Item,
//     >;

//     fn into_iter(self) -> Self::IntoIter {
//         self.types
//             .iter()
//             .enumerate()
//             .map(|(i, typ)| (TypeId::new(i as u32), *typ))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_typ_ids() {
        let mut typs = Typs::new();

        // make sure that error() and insert(Type::Error) return the same id
        assert_eq!(typs.error(), typs.insert(Type::Error));

        // make sure that atom(...) and insert(Type::Atom(...)) return the same id
        for atom in AtomType::ALL {
            assert_eq!(typs.atom(*atom), typs.insert(Type::Atom(*atom)));
        }
    }
}
