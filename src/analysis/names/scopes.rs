use crate::{
    strings::StrId,
    tir::{def::DefId, typ::TypId},
};
use std::collections::HashMap;

struct Scope {
    defs: HashMap<StrId, DefId>,
    typs: HashMap<StrId, TypId>,
}
impl Scope {
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
            typs: HashMap::new(),
        }
    }
    pub fn insert_def(&mut self, string: StrId, def: DefId) -> Result<(), DefId> {
        let prev = self.defs.insert(string, def);
        if let Some(prev) = prev {
            Err(prev)
        } else {
            Ok(())
        }
    }
    pub fn insert_typ(&mut self, string: StrId, typ: TypId) -> Result<(), TypId> {
        let prev = self.typs.insert(string, typ);
        if let Some(prev) = prev {
            Err(prev)
        } else {
            Ok(())
        }
    }
}

pub struct ScopeStack {
    stack: Vec<Scope>,
}
impl ScopeStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }
    pub fn push_scope(&mut self) {
        self.stack.push(Scope::new());
    }
    pub fn pop_scope(&mut self) {
        self.stack.pop();
    }
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn insert_def(&mut self, string: StrId, def_id: DefId) -> Result<(), DefId> {
        self.stack.last_mut().unwrap().insert_def(string, def_id)
    }
    pub fn insert_typ(&mut self, string: StrId, typ_id: TypId) -> Result<(), TypId> {
        self.stack.last_mut().unwrap().insert_typ(string, typ_id)
    }
    pub fn get_def(&self, string: StrId) -> Option<DefId> {
        for scope in self.stack.iter().rev() {
            if let Some(def_id) = scope.defs.get(&string) {
                return Some(*def_id);
            }
        }
        None
    }
    pub fn get_typ(&self, string: StrId) -> Option<TypId> {
        for scope in self.stack.iter().rev() {
            if let Some(typ_id) = scope.typs.get(&string) {
                return Some(*typ_id);
            }
        }
        None
    }
}
