use super::error::Result;

// Container: I: Id, V: Value
pub trait Container<'a, E> {
    type I: Copy;
    type IdIterator: Iterator<Item=Self::I>;

    fn insert(&mut self, element: E) -> Result<Self::I>{
        let id = try!(self.reserve());
        let _ = try!(self.update(id, element));
        Ok(id)
    }

    fn reserve(&mut self) -> Result<Self::I>;

    fn clear(&mut self) -> Result<()>;

    fn contains(&mut self, id: Self::I) -> Result<bool>;

    fn ids(&'a mut self) -> Self::IdIterator;

    fn remove(&mut self, id: Self::I) -> Result<Option<E>>;

    fn update(&mut self, id: Self::I, new_element:E) -> Result<Option<E>>;

}

pub trait BufferContainer<'a, E>: Container<'a, E> {
    fn flush(&mut self, id: Self::I);
    fn unfix(&mut self, id: Self::I);
    fn fix(&mut self, id: Self::I);
}

pub trait CloneContainer<'a, E>: Container<'a, E> {
    fn get_clone(&mut self, id: Self::I) -> Result<Option<E>>;
}


pub trait EntryContainer<'a, E>: Container<'a, E> {
    type VacantEntry: VacantEntry<Id=Self::I, Value=E>;
    type OccupiedEntry: OccupiedEntry<Id=Self::I, Value=E>;
    fn entry(&'a mut self, id: Self::I) -> Result<Entry<Self::OccupiedEntry, Self::VacantEntry>>;
}

/// A View into a single occupied location in a HashMap
pub trait OccupiedEntry {
    type Id;
    type Value;
    fn id(&self) -> Self::Id;
    fn get(&self) -> &Self::Value;
    // fn get_mut(&mut self) -> &mut V;
    // fn into_mut(self) -> &'a mut V;
    fn update(&mut self, value: Self::Value) -> Self::Value;
    fn remove(self) -> Self::Value;
}

/// A View into a single empty location in a HashMap
pub trait VacantEntry{
    type Id;
    type Value;
    fn id(&self) -> Self::Id;

    fn insert(self, value: Self::Value);
}

/// A View into a single location
#[derive(Debug)]
pub enum Entry<O: OccupiedEntry, V: VacantEntry> {
    /// An occupied View
    Occupied(O),
    /// A vacant View
    Vacant(V),
}
