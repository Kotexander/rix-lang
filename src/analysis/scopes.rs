use crate::{strings::StrId, tir};
use std::collections::HashMap;

struct Scope {
    defs: HashMap<StrId, tir::def::DefId>,
    typs: HashMap<StrId, tir::typ::TypeId>,
}
impl Scope {
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
            typs: HashMap::new(),
        }
    }
    pub fn insert_def(
        &mut self,
        string: StrId,
        def_id: tir::def::DefId,
    ) -> Result<(), tir::def::DefId> {
        let old = self.defs.insert(string, def_id);
        if let Some(old) = old {
            Err(old)
        } else {
            Ok(())
        }
    }
    pub fn insert_typ(
        &mut self,
        string: StrId,
        typ_id: tir::typ::TypeId,
    ) -> Result<(), tir::typ::TypeId> {
        let old = self.typs.insert(string, typ_id);
        if let Some(old) = old {
            Err(old)
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
    pub fn insert_def(
        &mut self,
        string: StrId,
        def_id: tir::def::DefId,
    ) -> Result<(), tir::def::DefId> {
        if let Some(bindings) = self.stack.last_mut() {
            bindings.insert_def(string, def_id)
        } else {
            panic!();
        }
    }
    pub fn insert_typ(
        &mut self,
        string: StrId,
        typ_id: tir::typ::TypeId,
    ) -> Result<(), tir::typ::TypeId> {
        if let Some(bindings) = self.stack.last_mut() {
            bindings.insert_typ(string, typ_id)
        } else {
            panic!();
        }
    }
    pub fn get_def(&self, string: StrId) -> Option<tir::def::DefId> {
        for scope in self.stack.iter().rev() {
            if let Some(def_id) = scope.defs.get(&string) {
                return Some(*def_id);
            }
        }
        None
    }
    pub fn get_typ(&self, string: StrId) -> Option<tir::typ::TypeId> {
        for scope in self.stack.iter().rev() {
            if let Some(typ_id) = scope.typs.get(&string) {
                return Some(*typ_id);
            }
        }
        None
    }
}
