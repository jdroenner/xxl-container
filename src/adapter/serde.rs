use bincode;
use serde;

use std::marker::{PhantomData};

use container::{Container, CloneContainer};
use super::super::error::{Result};



pub struct SerdeAdapter<E, C> {
    container: C,
    phantom: PhantomData<E>,
}

impl <E, C> SerdeAdapter<E, C> {
    pub fn new(container: C) -> SerdeAdapter<E, C> {
        SerdeAdapter{container: container, phantom: PhantomData}
    }
}

impl<'a, AE, C > Container<'a, AE> for SerdeAdapter<AE, C> where AE: serde::ser::Serialize + serde::de::Deserialize,  C: Container<'a, Vec<u8> > {
    type I = C::I;
    type IdIterator = C::IdIterator;

    fn ids(&'a mut self) -> Self::IdIterator {
        self.container.ids()
    }


    fn reserve(&mut self) -> Result<Self::I> {
        self.container.reserve()
    }

    fn clear(&mut self) -> Result<()>{
        self.container.clear()
    }

    fn contains(&mut self, id: Self::I) -> Result<bool>{
        self.container.contains(id)
    }

    fn update(&mut self, id: Self::I, new_element: AE) -> Result<Option<AE>>{
        let new_serialized = try!(bincode::serde::serialize(&new_element, bincode::SizeLimit::Infinite));
        if let Some(old) = try!(self.container.update(id, new_serialized)){
            let old_deserialized = try!(bincode::serde::deserialize(&old));
            Ok(Some(old_deserialized))
        }
        else {
            Ok(None)
        }
    }

    fn remove(&mut self, id: Self::I ) -> Result<Option<AE>> {
        if let Some(old_element) = try!(self.container.remove(id)){
            let old_deserialized = try!(bincode::serde::deserialize(&old_element));
            Ok(Some(old_deserialized))
        }
        else{
            Ok(None)
        }
    }
}

impl<'a, AE, C > CloneContainer<'a, AE> for SerdeAdapter<AE, C> where AE: serde::ser::Serialize + serde::de::Deserialize,  C: CloneContainer<'a, Vec<u8> > {
    fn get_clone(&mut self, id: Self::I) -> Result<Option<AE>> {
        if let Some(old_element) = try!(self.container.get_clone(id)){
            let old_deserialized = try!(bincode::serde::deserialize(&old_element));
            Ok(Some(old_deserialized))
        }
        else{
            Ok(None)
        }
    }
}
