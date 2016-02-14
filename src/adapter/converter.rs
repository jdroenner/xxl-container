use container::{Container, CloneContainer};
use super::super::error::{Result};

pub struct ConverterAdapter<A, B, C>{
    into_fn: A,
    from_fn: B,
    container: C,
}

impl<A, B, C> ConverterAdapter<A, B, C>{
    pub fn new(into_fn: A, from_fn: B, container: C) -> ConverterAdapter<A,B,C> {
        ConverterAdapter{into_fn:into_fn, from_fn:from_fn, container:container}
    }
}

impl<'a, A, B, AE, BE, C > Container<'a, AE> for ConverterAdapter<A, B, C> where A: Fn(AE) -> BE, B: Fn(BE) -> AE, C: Container<'a, BE> {
    type I = C::I;
    type IdIterator = C::IdIterator;

    fn ids(&'a mut self) -> Self::IdIterator {
        self.container.ids()
    }

    fn remove(&mut self, id: Self::I ) -> Result<Option<AE>> {
        if let Some(be) = try!(self.container.remove(id)){
            let ae = (self.from_fn)(be);
            return Ok(Some(ae))
        }
        else{
            Ok(None)
        }
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
        let new_converted = (self.into_fn)(new_element);
        if let Some(old_element) = try!(self.container.update(id, new_converted)){
            Ok(Some((self.from_fn)(old_element)))
        }
        else {
            Ok(None)
        }
    }

}

impl <'a, A, B, AE, BE, C> CloneContainer<'a, AE> for ConverterAdapter<A, B, C> where A: Fn(AE) -> BE, B: Fn(BE) -> AE, C: CloneContainer<'a, BE> {
    fn get_clone(&mut self, id: Self::I) -> Result<Option<AE>> {
        if let Some(be) = try!(self.container.get_clone(id)){
            let ae = (self.from_fn)(be);
            return Ok(Some(ae))
        }
        else{
            Ok(None)
        }
    }
}
