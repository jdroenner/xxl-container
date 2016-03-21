use container::{Container, CloneContainer};
use std::{mem};
use error::{Result, ContainerError};

pub type Id = usize;

#[derive(Debug)]
pub struct VecContainer<E> {
    vec: Vec<Slot<E>>,
    free_list: Vec<Id>,
}

#[derive(Debug)]
enum Slot<E> {
    Free,
    Reserved,
    Occupied(E),
}


impl<'a, E:'a > Container<'a, E> for VecContainer<E> {
    type I = Id;
    type IdIterator = Box<Iterator<Item=Self::I> + 'a>;

    fn reserve(&mut self) -> Result<Id> {
        while let Some(free_id) = self.free_list.pop() {
            if let Some(slot) = self.vec.get_mut(free_id) {
                let mut temp = Slot::Reserved;
                mem::swap(slot, &mut temp);
                return Ok(free_id);
            }

        }

        self.vec.push(Slot::Reserved);
        Ok(self.vec.len() - 1)
    }

    fn update(&mut self, id: Id, element: E) -> Result<Option<E>> {
        if let Some(slot) = self.vec.get_mut(id){
            let mut temp = Slot::Occupied(element);
            {mem::swap(slot, &mut temp);}
            match temp {
                Slot::Reserved => return Ok(None),
                Slot::Occupied(element) => return Ok(Some(element)),
                Slot::Free => return Err(ContainerError::InvalidId),
            }
        }
        Err(ContainerError::InvalidId)
    }

    fn clear(&mut self) -> Result<()>{
        self.vec.clear();
        self.free_list.clear();
        Ok(())
    }

    /// Remove an element from the container
    fn remove(&mut self, id: Id) -> Result<Option<E>> {
        let mut temp = Slot::Free;

        if let Some(slot) = self.vec.get_mut(id) {
            mem::swap(slot, &mut temp);
            self.free_list.push(id);
        }
        else {
            return Err(ContainerError::InvalidId);
        }

        /* This is not really usefull as a Vec will only shrink if it is called explicitly.
        while let Some(&Slot::Free) = self.vec.last() {
                let _ = self.vec.pop();
        }
        */

        match temp {
            Slot::Reserved => return Ok(None),
            Slot::Occupied(element) => return Ok(Some(element)),
            Slot::Free => return Err(ContainerError::InvalidId),
        }
    }

    fn contains(&mut self, id: Id) -> Result<bool> {
        Ok(self.contains_element(id))
    }

    fn ids(&'a mut self) -> Self::IdIterator{
        Box::new(self.vec.iter().enumerate().filter(|&(_, slot)| {
                return match *slot  {
                   Slot::Occupied(_) => true,
                   Slot::Reserved => true,
                   _ => false,
               }
        }).map(|(index, _)| index))
    }
}


impl<E> VecContainer<E> {
    pub fn with_capacity(capacity: usize) -> Self {
        VecContainer {
            vec: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    pub fn new() -> Self {
        VecContainer {
            vec: Vec::new(),
            free_list: Vec::new(),
        }
    }



    pub fn contains_element(&self, id: Id) -> bool {
        if let Some(slot) = self.vec.get(id) {
            let contains = match *slot  {
                Slot::Occupied(_) => true,
                Slot::Reserved => true,
                Slot::Free => false,
            };
            return contains;
        }
        false
    }



    pub fn get<'a>(&'a self, id: Id) -> Option<&'a E> {
        if let Some(&Slot::Occupied(ref element)) = self.vec.get(id) {
            return Some(&element);
        }
        None
    }

    pub fn get_mut<'a>(&'a mut self, id: Id) -> Option<&'a mut E> {
        if let Some(&mut Slot::Occupied(ref mut element)) = self.vec.get_mut(id) {
            return Some(element);
        }
        None
    }

    pub fn free_slots(&self) -> usize {
        self.free_list.len()
    }

}

impl <'a, E:'a> CloneContainer<'a, E> for VecContainer<E> where E: Clone {
    fn get_clone(&mut self, id: Self::I) -> Result<Option<E>> {
        Ok(self.get(id).cloned())
    }
}


/*
pub struct VecContainerEntry<'a, E: 'a> {
    container: &'a mut VecContainer<E>,
    element_id: Id,
}

impl<'a, E> VacantEntry for VecContainerEntry<'a, E> {
    type Id = Id;
    type Value = E;

    fn id(&self) -> Id {
        self.element_id
    }

    fn insert(self, value: Self::Value) {}
}

impl<'a, E> OccupiedEntry for VecContainerEntry<'a, E> {
    type Id = Id;
    type Value = E;

    fn id(&self) -> Id {
        self.element_id
    }

    fn get(&self) -> &Self::Value {
        self.container.vec[self.element_id].as_ref().unwrap()
    }
    // fn get_mut(&mut self) -> &mut V;
    // fn into_mut(self) -> &'a mut V;
    fn update(&mut self, value: Self::Value) -> Self::Value {
        let mut temp = Some(value);
        mem::swap(&mut temp, &mut self.container.vec[self.element_id]);
        temp.expect("OccupiedEntry contained no value")
    }
    fn remove(self) -> Self::Value {
        self.container.vec.remove(self.element_id).unwrap()
    }
}
*/

//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_capacity() {
        let vec_c: VecContainer<u64> = VecContainer::with_capacity(10);
        assert_eq!(10, vec_c.vec.capacity());
    }

    #[test]
    fn reserve(){
        use container::Container;
        let mut vec_c:VecContainer<u64> = VecContainer::with_capacity(1);
        let id = vec_c.reserve().unwrap();
        assert_eq!(id, 0);
    }

    #[test]
    fn insert() {
        use container::Container;
        let mut vec_c = VecContainer::with_capacity(10);
        let id = vec_c.insert(1.1).unwrap();
        assert_eq!(0, id);
        assert_eq!(1, vec_c.vec.len());
        assert_eq!(0, vec_c.free_slots());
    }

    #[test]
    fn remove() {
        use container::Container;
        let mut vec_c = VecContainer::with_capacity(10);
        let one = vec_c.insert(1.1).unwrap();
        let element = vec_c.remove(one).unwrap();
        assert_eq!(element, Some(1.1));
        assert_eq!(1, vec_c.free_slots());

        // insert two elements remove one element
        let two = vec_c.insert(2.2).unwrap();
        let _ = vec_c.insert(3.3);
        let two_element = vec_c.remove(two).unwrap();
        assert_eq!(two_element, Some(2.2));
        assert_eq!(1, vec_c.free_slots());
        assert_eq!(2, vec_c.vec.len());
    }

}
