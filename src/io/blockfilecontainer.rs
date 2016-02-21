use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Seek, SeekFrom, Result as IoResult, Read, Write};
use std::path::Path;
use container::{Container, VacantEntry, OccupiedEntry, CloneContainer};
use super::super::error::{ContainerError, Result};

use super::bitsetfile::{BitSetFile, ContainsIterator};
use super::stackfile::StackFile;

pub type Id = u64;
pub type Block = Vec<u8>; //TODO: this should probably be [u8, N] when N is generic...

// pub type BlockFileContainerEntry<'a> = Entry<OccupiedBlockFileContainerEntry<'a>,
//                                              VacantBlockFileContainerEntry<'a>>;
//
// #[derive(Debug)]
// pub struct VacantBlockFileContainerEntry<'a> {
//     bfc: &'a mut BlockFileContainer,
//     id: Id,
// }
//
// impl<'a> VacantEntry for VacantBlockFileContainerEntry<'a> {
//     type Id = Id;
//     type Value = Block;
//     fn insert(self, element: Block){
//         write_block(&mut self.bfc.container_file, self.id as u64, &element, self.bfc.block_size);
//     }
//
//     fn id(&self) -> Id {
//         self.id
//     }
// }
//
// #[derive(Debug)]
// pub struct OccupiedBlockFileContainerEntry<'a> {
//     bfc: &'a mut BlockFileContainer,
//     id: Id,
//     block: Block,
// }
//
// impl<'a> OccupiedEntry for OccupiedBlockFileContainerEntry<'a> {
//     type Id = Id;
//     type Value = Block;
//
//     fn id(&self) -> Id {
//         self.id
//     }
//
//     fn get(&self) -> &Block {
//         &self.block
//     }
//     fn update(&mut self, element: Block) -> Block {
//         let mut temp = element;
//         mem::swap(&mut self.block, &mut temp);
//         write_block(&mut self.bfc.container_file, self.id as u64, &self.block, self.bfc.block_size);
//         temp
//     }
//     fn remove(self) -> Block {
//         self.bfc.remove(self.id);
//         self.block
//     }
// }

fn write_block(file: &mut File, offset: u64, element: &Block, block_size: usize) -> IoResult<usize> {
    if element.len() > block_size{
        return Err(Error::new(ErrorKind::InvalidInput, "Block size exceeds the containers block_size"));
    }
    let position = offset * block_size as u64;
    try!(file.seek(SeekFrom::Start(position)));
    let bytes_written = try!(file.write(element));
    //println!("write_block: [{}] = {}", offset, bytes_written);
    assert!(bytes_written > 0);
    assert!(bytes_written <= block_size);
    Ok(bytes_written)
}

fn read_block(file: &mut File, offset: u64, block_size: usize) -> IoResult<Block> {
    let block_pos = offset * block_size as u64;
    try!(file.seek(SeekFrom::Start(block_pos)));
    let mut buffer = vec![0;block_size];
    let bytes_read = try!(file.read(&mut buffer));
    assert!(bytes_read > 0);
    assert!(bytes_read <= block_size);
    Ok(buffer)
}

pub struct BlockFileContainer {
    container_file: File,
    metadata_file: File,
    reserved_bit_map: BitSetFile,
    used_bit_map: BitSetFile,
    free_list_file: StackFile,
    block_size: usize,
}

impl BlockFileContainer {

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn size(&self) -> u64 {
        self.reserved_bit_map.size()
    }

    pub fn reserve(&mut self) -> Result<Id> {
        let mut id: Id;
        if let Some(max_id) = self.reserved_bit_map.max_bit(){
            id = max_id as Id + 1;
            while let Some(free_id) = try!(self.free_list_file.next()) {
                if free_id < (max_id) {
                    id = free_id as Id;
                    break;
                }
            }
            //println!("[reserve] new free id: {}", id);
        }
        else {
            id = 0;
        }
        try!(self.reserved_bit_map.insert(id as u64));
        //println!("[reserve] reserved new ID {}", id);
        Ok(id)
    }

    pub fn remove(&mut self, id: Id) -> Result<Option<Block>>{

        let reserved_was_set = try!(self.reserved_bit_map.remove(id as u64));
        //println!("[remove] id: {}, reserved_was_set: {}",id, reserved_was_set);

        if reserved_was_set {
            let used_was_set = try!(self.used_bit_map.remove(id as u64));
            if used_was_set {
                let old_block = try!(read_block(&mut self.container_file, id as u64, self.block_size));

                if self.used_bit_map.is_empty() {
                    try!(self.free_list_file.clear());
                    try!(self.container_file.set_len(0));
                }
                else {
                    let _ = try!(self.free_list_file.add(id as u64));
                    //println!("[remove] free_elements: {}", free_elements);
                    if let Some(new_max_used_id) = self.used_bit_map.max_bit(){ //TODO: Investigate if setting the len of the container_file everytime a used block is removed is a bad idea.
                        try!(self.container_file.set_len((new_max_used_id+1)*self.block_size as u64));
                    }
                }
                Ok(Some(old_block))
            }
            else{
                //println!("[remove] reserved_was_set: {}, used_was_set: {}",reserved_was_set, used_was_set);
                return Ok(None)
            }
        }
        else{
            Err(ContainerError::InvalidId)
        }
    }

    pub fn update(&mut self, id: Id, element: Block) -> Result<Option<Block>> { //TODO: reading the old value for every update my be a performance issue!
        let reserved_was_set = try!(self.reserved_bit_map.contains(id as u64));

        if reserved_was_set {
            let used_was_not_set = try!(self.used_bit_map.insert(id as u64));
            //println!("[update] reserved_was_set: {}, used_was_not_set: {}",reserved_was_set, used_was_not_set);

            let old_block;
            if !used_was_not_set {
                old_block = Some(try!(read_block(&mut self.container_file, id as u64, self.block_size)));
            }
            else{
                old_block = None;
            }
            try!(write_block(&mut self.container_file, id as u64, &element, self.block_size));

            return Ok(old_block);
        }
        Err(ContainerError::InvalidId)
    }

    pub fn get(&mut self, id: Id) -> Result<Option<Block>>{
        if try!(self.contains(id)) && try!(self.used_bit_map.contains(id as u64)) {
            let old_block = try!(read_block(&mut self.container_file, id as u64, self.block_size));
            return Ok(Some(old_block));
        }
        Ok(None)
    }

    pub fn contains(&mut self, id: Id) -> Result<bool> {
        let contains = try!(self.reserved_bit_map.contains(id));
        Ok(contains)
    }

    pub fn clear(&mut self) -> Result<()>{
        try!(self.free_list_file.clear());
        try!(self.used_bit_map.clear());
        try!(self.reserved_bit_map.clear());
        try!(self.container_file.set_len(0));
        Ok(())
    }

    pub fn reserved_ids(&mut self) -> ContainsIterator {
        self.reserved_bit_map.contains_iter()
    }

    pub fn used_ids(&mut self) -> ContainsIterator{
        self.used_bit_map.contains_iter()
    }

    pub fn new_from_prefix_and_block_size(prefix: &Path, block_size: usize) -> IoResult<BlockFileContainer> {
        let container_file = try!(OpenOptions::new().read(true).write(true).create(true).open(prefix.with_extension("ctr")));
        let metadata_file = try!(OpenOptions::new().read(true).write(true).create(true).open(prefix.with_extension("mtd")));
        let reserved_bit_map_file = try!(OpenOptions::new().read(true).write(true).create(true).open(prefix.with_extension("rbm")));
        let updated_bit_map_file = try!(OpenOptions::new().read(true).write(true).create(true).open(prefix.with_extension("ubm")));
        let free_list_file = try!(OpenOptions::new().read(true).write(true).create(true).open(prefix.with_extension("flt")));

        Ok(BlockFileContainer {
            container_file: container_file,
            metadata_file: metadata_file,
            reserved_bit_map: BitSetFile::new(reserved_bit_map_file),
            used_bit_map: BitSetFile::new(updated_bit_map_file),
            free_list_file: StackFile::new(free_list_file),
            block_size: block_size,
        })
    }
}

impl Drop for BlockFileContainer {
    fn drop(&mut self){
        self.metadata_file.sync_data().expect("could not write metadata_file");
    }
}

impl<'a> Container<'a, Block> for BlockFileContainer {
    type I = Id;
    type IdIterator = ContainsIterator<'a>;
    //type VacantEntry = VacantBlockFileContainerEntry<'a>;
    //type OccupiedEntry = OccupiedBlockFileContainerEntry<'a>;


    fn reserve(&mut self) -> Result<Id> {
        self.reserve()
    }

    fn clear(&mut self) -> Result<()> {
        self.clear()
    }


    fn contains(&mut self, id: Id) -> Result<bool> {
        self.contains(id)
    }

    fn ids(&'a mut self) -> ContainsIterator<'a> {
        self.reserved_ids()
    }

    fn update(&mut self, id: Id, new_element: Block) -> Result<Option<Block>>{
        self.update(id, new_element)
    }

    /*
    fn entry(&'a mut self, id: Id) -> Result<BlockFileContainerEntry<'a>> {

        if try!(self.contains(id)) {
            return Ok(Entry::Occupied(OccupiedBlockFileContainerEntry{
                id: id,
                block: try!(read_block(&mut self.container_file, id as u64, self.block_size)),
                bfc: self,
            }));
        }

        Ok(Entry::Vacant(VacantBlockFileContainerEntry {bfc: self, id: id }))
    }
    */

    fn remove(&mut self, id: Id) -> Result<Option<Block>> {
        self.remove(id)
    }
}

impl<'a> CloneContainer<'a, Block> for BlockFileContainer {
    fn get_clone(&mut self, id: Self::I) -> Result<Option<Block>>{
        self.get(id)
    }
}


#[test]
fn blockfilecontainer_reserve() {
    let prefix = Path::new("./test_output/bct_test_reserve");
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(&prefix, 8).unwrap();
    let id = bfc.reserve().unwrap();
    assert_eq!(id, 0);
    assert_eq!(bfc.size(), 1);
}

#[test]
fn blockfilecontainer_reserve_1000() {
    let prefix = Path::new("./test_output/bct_test_reserve_1000");
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(&prefix, 8).unwrap();
    for i in  0 .. 1000 {
        let id = bfc.reserve().unwrap();
        assert_eq!(id, i);
        assert_eq!(bfc.size(), id + 1);
    }
}

#[test]
fn blockfilecontainer_insert() {
    let prefix = Path::new("./test_output/bct_test_insert");
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(&prefix, 8).unwrap();
    let id = bfc.insert(vec!(1u8,2,3,4,5,6,7,8)).unwrap();
    assert_eq!(id, 0);
    assert_eq!(bfc.size(), 1);
    assert_eq!(bfc.contains(id).unwrap(), true);
    let block = read_block(&mut bfc.container_file, 0, bfc.block_size).unwrap();
    assert_eq!(block, vec!(1u8,2,3,4,5,6,7,8))
}

#[test]
fn blockfilecontainer_insert_256() {
    let prefix = Path::new("./test_output/bct_test_insert_256");
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(&prefix, 8).unwrap();
    for i in  0u8 .. 255 {
        let id = bfc.insert(vec!(i,i,i,i,i,i,i,i)).unwrap();
        assert_eq!(id, i as Id);
        assert_eq!(bfc.size(), id + 1);
        let block = read_block(&mut bfc.container_file, id as u64, bfc.block_size).unwrap();
        assert_eq!(block, vec!(i,i,i,i,i,i,i,i));
        assert_eq!(bfc.contains(i as Id).unwrap(), true);
    }
}

#[test]
fn blockfilecontainer_remove() {
    let prefix = Path::new("./test_output/bct_test_remove");
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(&prefix, 8).unwrap();
    let id = bfc.insert(vec!(1u8,2,3,4,5,6,7,8)).unwrap();
    assert_eq!(id, 0);
    assert_eq!(bfc.size(), 1);
    let block = bfc.remove(id).unwrap();
    assert_eq!(block, Some(vec!(1u8,2,3,4,5,6,7,8)));
    assert_eq!(bfc.size(), 0);
}

#[test]
fn blockfilecontainer_remove_256() {
    let prefix = Path::new("./test_output/bct_test_remove_256");
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(&prefix, 8).unwrap();
    for i in  0u8 .. 255 {
        let id = bfc.insert(vec!(i,i,i,i,i,i,i,i)).unwrap();
        assert_eq!(id, i as Id);
        assert_eq!(bfc.size(), id + 1);
    }
    for i in  0u8 .. 255 {
        let block = bfc.remove(i as Id).unwrap();
        assert_eq!(block, Some(vec!(i,i,i,i,i,i,i,i)));
        assert_eq!(bfc.contains(i as Id).unwrap(), false);
    }
    assert_eq!(bfc.size(), 0);
}
