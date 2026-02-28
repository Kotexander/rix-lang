#[derive(Debug)]
#[repr(transparent)]
pub struct ArenaId<T>(u32, std::marker::PhantomData<T>);
impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ArenaId<T> {}
impl<T> PartialEq for ArenaId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for ArenaId<T> {}
impl<T> std::hash::Hash for ArenaId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Arena<T> {
    items: Vec<T>,
}
impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn alloc(&mut self, item: T) -> ArenaId<T> {
        let id = self.items.len() as u32;
        self.items.push(item);
        ArenaId(id, std::marker::PhantomData)
    }
    pub fn get(&self, id: ArenaId<T>) -> Option<&T> {
        self.items.get(id.0 as usize)
    }
}
impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = (ArenaId<T>, &'a T);
    type IntoIter = std::iter::Map<
        std::iter::Enumerate<std::slice::Iter<'a, T>>,
        fn((usize, &'a T)) -> Self::Item,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| (ArenaId(i as u32, std::marker::PhantomData), item))
    }
}
impl<T> std::ops::Index<ArenaId<T>> for Arena<T> {
    type Output = T;
    fn index(&self, id: ArenaId<T>) -> &Self::Output {
        &self.items[id.0 as usize]
    }
}
