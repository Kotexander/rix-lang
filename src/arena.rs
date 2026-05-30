use std::num::NonZeroU32;

#[repr(transparent)]
pub struct ArenaId<T>(NonZeroU32, std::marker::PhantomData<T>);
impl<T> ArenaId<T> {
    pub fn new(id: u32) -> Self {
        ArenaId(
            unsafe { NonZeroU32::new_unchecked(id + 1) },
            std::marker::PhantomData,
        )
    }
    #[inline]
    pub fn get(self) -> u32 {
        self.0.get() - 1
    }

    #[inline]
    pub fn idx(self) -> usize {
        self.get() as usize
    }
}
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
impl<T> std::fmt::Debug for ArenaId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", std::any::type_name::<T>(), self.get())
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
        ArenaId::new(id)
    }
    pub fn alloc_iter(&mut self, items: impl IntoIterator<Item = T>) -> ArenaRange<T> {
        let start = self.items.len() as u32;
        self.items.extend(items);
        let end = self.items.len() as u32;
        ArenaRange::new(ArenaId::new(start), ArenaId::new(end))
    }
    pub fn get(&self, id: ArenaId<T>) -> Option<&T> {
        self.items.get(id.idx())
    }
    pub fn get_mut(&mut self, id: ArenaId<T>) -> Option<&mut T> {
        self.items.get_mut(id.idx())
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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
            .map(|(i, item)| (ArenaId::new(i as u32), item))
    }
}
impl<T> std::ops::Deref for Arena<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.items.as_slice()
    }
}
impl<T> std::ops::Index<ArenaId<T>> for Arena<T> {
    type Output = T;
    fn index(&self, id: ArenaId<T>) -> &Self::Output {
        &self.items[id.idx()]
    }
}
impl<T> std::ops::Index<ArenaRange<T>> for Arena<T> {
    type Output = [T];
    fn index(&self, range: ArenaRange<T>) -> &Self::Output {
        &self.items[range.start as usize..range.end as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaRange<T> {
    start: u32,
    end: u32,
    _marker: std::marker::PhantomData<T>,
}
impl<T> ArenaRange<T> {
    pub fn new(start: ArenaId<T>, end: ArenaId<T>) -> Self {
        Self {
            start: start.get(),
            end: end.get(),
            _marker: std::marker::PhantomData,
        }
    }
    // pub fn new_empty() -> Self {
    //     Self {
    //         start: 0,
    //         end: 0,
    //         _marker: std::marker::PhantomData,
    //     }
    // }
    pub fn len(&self) -> u32 {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = ArenaId<T>> {
        (self.start..self.end).map(ArenaId::new)
    }
}
