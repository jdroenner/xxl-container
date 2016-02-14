use bincode;
use rustc_serialize;

use std::marker::{PhantomData};

use container::{Container, CloneContainer};
use super::super::error::{Result};

pub struct SerializeAdapter<E, C> {
    container: C,
    phantom: PhantomData<E>,
}

impl <E, C> SerializeAdapter<E, C> {
    pub fn new(container: C) -> SerializeAdapter<E, C> {
        SerializeAdapter{container: container, phantom: PhantomData}
    }
}

impl<'a, AE, C > Container<'a, AE> for SerializeAdapter<AE, C> where AE: rustc_serialize::Decodable + rustc_serialize::Encodable,  C: Container<'a, Vec<u8> > {
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
        let new_serialized = try!(bincode::rustc_serialize::encode(&new_element, bincode::SizeLimit::Infinite));
        if let Some(old_element) = try!(self.container.update(id, new_serialized)){
            let old_deserialized = try!(bincode::rustc_serialize::decode(&old_element));
            Ok(Some(old_deserialized))
        }
        else {
            Ok(None)
        }
    }

    fn remove(&mut self, id: Self::I ) -> Result<Option<AE>> {
        if let Some(old_element) = try!(self.container.remove(id)){
            let old_deserialized = try!(bincode::rustc_serialize::decode(&old_element));
            Ok(Some(old_deserialized))
        }
        else{
            Ok(None)
        }
    }
}

impl<'a, AE, C > CloneContainer<'a, AE> for SerializeAdapter<AE, C> where AE: rustc_serialize::Decodable + rustc_serialize::Encodable,  C: CloneContainer<'a, Vec<u8> > {
    fn get_clone(&mut self, id: Self::I) -> Result<Option<AE>> {
        if let Some(old_element) = try!(self.container.get_clone(id)){
            let old_deserialized = try!(bincode::rustc_serialize::decode(&old_element));
            Ok(Some(old_deserialized))
        }
        else{
            Ok(None)
        }
    }
}
