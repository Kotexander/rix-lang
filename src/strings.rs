use std::{collections::HashMap, num::NonZeroU32, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StrId(NonZeroU32);
impl StrId {
    pub fn new(id: u32) -> Self {
        StrId(unsafe { NonZeroU32::new_unchecked(id + 1) })
    }
    #[inline]
    pub fn get(self) -> u32 {
        self.0.get() - 1
    }
}

#[derive(Debug, Default)]
pub struct Interner {
    strings: Vec<Arc<str>>,
    map: HashMap<Arc<str>, StrId>,
}
impl Interner {
    pub fn intern(&mut self, string: impl AsRef<str>) -> StrId {
        let string = string.as_ref();
        if let Some(id) = self.map.get(string) {
            *id
        } else {
            let id = self.strings.len() as u32;
            let arc_str = Arc::<str>::from(string);
            self.strings.push(arc_str.clone());
            let string_id = StrId::new(id);
            self.map.insert(arc_str, string_id);
            string_id
        }
    }

    pub fn contains(&self, string: &str) -> Option<StrId> {
        self.map.get(string).copied()
    }

    pub fn resolve(&self, string: StrId) -> &str {
        self.strings[string.get() as usize].as_ref()
    }
}
