use std::fs::{File};
use std::io::{Result, Seek, SeekFrom};
use std::{cmp};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

#[inline]
pub fn bit_map_offset_and_bit_mask(bit: u64) -> (u64, u8){
    let offset = bit/8; //(std::u8::BITS);
    let mask = 1<<(bit%8); //(std::u8::BITS));
    (offset, mask)
}

#[test]
fn bit_map_offset_and_bit_mask_test() {
    assert_eq!(bit_map_offset_and_bit_mask(0), (0, 0b00000001));
    assert_eq!(bit_map_offset_and_bit_mask(1), (0, 0b00000010));
    assert_eq!(bit_map_offset_and_bit_mask(7), (0, 0b10000000));
    assert_eq!(bit_map_offset_and_bit_mask(8), (1, 0b00000001));
    assert_eq!(bit_map_offset_and_bit_mask(9), (1, 0b00000010));
    assert_eq!(bit_map_offset_and_bit_mask(15), (1, 0b10000000));
    assert_eq!(bit_map_offset_and_bit_mask(16), (2, 0b00000001));
    assert_eq!(bit_map_offset_and_bit_mask(17), (2, 0b00000010));
    assert_eq!(bit_map_offset_and_bit_mask(23), (2, 0b10000000));
    assert_eq!(bit_map_offset_and_bit_mask(24), (3, 0b00000001));
    assert_eq!(bit_map_offset_and_bit_mask(25), (3, 0b00000010));
    assert_eq!(bit_map_offset_and_bit_mask(31), (3, 0b10000000));
}

#[inline]
pub fn max_id_from(offset: u64, mask: u8) -> Option<u64> {
    if offset == 0 && mask == 0 {
        None
    } else if mask == 0 {
        Some(offset - 1)
    } else {
        let base_id = offset * 8; //(std::u8::BITS);
        let max_bit = 7 - mask.leading_zeros();
        Some(base_id + max_bit as u64)
    }
}

#[test]
fn max_id_from_test() {
    assert_eq!(max_id_from(0, 0b00000000), None);
    assert_eq!(max_id_from(0, 0b00000001), Some(0));
    assert_eq!(max_id_from(0, 0b00000010), Some(1));
    assert_eq!(max_id_from(0, 0b10000000), Some(7));
    assert_eq!(max_id_from(1, 0b00000001), Some(8));
    assert_eq!(max_id_from(1, 0b00000010), Some(9));
    assert_eq!(max_id_from(1, 0b10000000), Some(15));
    assert_eq!(max_id_from(2, 0b00000001), Some(16));
    assert_eq!(max_id_from(2, 0b00000010), Some(17));
    assert_eq!(max_id_from(2, 0b10000000), Some(23));
    assert_eq!(max_id_from(3, 0b00000001), Some(24));
    assert_eq!(max_id_from(3, 0b00000010), Some(25));
    assert_eq!(max_id_from(3, 0b10000000), Some(31));
    assert_eq!(max_id_from(4, 0b00000001), Some(32));
}

static  HEADER_BYTE_SIZE: u64 = 2*8;

#[derive(Debug)]
pub struct BitSetFile{
    file: File, //file_cell: RefCell<File>,
    max_bit: u64,
    size: u64,
}

impl BitSetFile {
    pub fn new(file: File) -> Self {
        let mut bsf = BitSetFile{file: file, max_bit: 0, size: 0};
        bsf.write_header().expect("Could not write file header");
        bsf
    }

    pub fn open(file: File) -> Self{
        let mut bsf = BitSetFile{file: file, max_bit: 0, size: 0};

        if let Ok(header) = bsf.read_header(){
            bsf.size = header.0;
            bsf.max_bit = header.1;
        }
        bsf
    }

    /// Adds a value to the set. Returns `true` if the value was not already present in the set.
    pub fn insert(&mut self, bit: u64) -> Result<bool>{
        let (bit_map_offset, mask) = bit_map_offset_and_bit_mask(bit);
        let _ = try!(self.file.seek(SeekFrom::Start(bit_map_offset + HEADER_BYTE_SIZE)));

        let was_set;
        if self.is_empty() || (bit > self.max_bit && mask == 1) {
            try!(self.file.write_u8(mask));
            was_set = false;
        }
        else {
            let byte = try!(self.file.read_u8());
            try!(self.file.seek(SeekFrom::Start(bit_map_offset + HEADER_BYTE_SIZE)));
            try!(self.file.write_u8(byte | mask));
            was_set = byte & mask == mask;
        }
        if !was_set {
            self.size+=1;
            self.max_bit = cmp::max(bit, self.max_bit);
        }
        //println!("[set_bit] id: {} was_set {}", bit, was_set);
        Ok(!was_set)
    }

    pub fn contains(&mut self, bit: u64) -> Result<bool>{
        if self.size() > 0 && /*0 <= bit &&*/ bit <= self.max_bit {
            let (bit_map_offset, mask) = bit_map_offset_and_bit_mask(bit);
            try!(self.file.seek(SeekFrom::Start(bit_map_offset + HEADER_BYTE_SIZE)));
            let byte = try!(self.file.read_u8());
            return Ok(byte & mask == mask);
        }
        Ok(false)
    }

    pub fn max_bit(&self) -> Option<u64> {
        if self.size() > 0 {
            Some(self.max_bit)
        }
        else {
            None
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) -> Result<()>{
        self.size = 0;
        self.max_bit = 0;
        self.file.set_len(HEADER_BYTE_SIZE)
    }

    pub fn remove(&mut self, bit: u64) -> Result<bool>{
        if !self.is_empty() && /*0 <= bit &&*/ bit <= self.max_bit {
            let (mut bit_map_offset, mask) = bit_map_offset_and_bit_mask(bit);
            //println!("[BitSetFile::unset_bit] ID: {} reserved_map_pos: {} mask: {:b}", bit, bit_map_offset, mask);
            let _ = try!(self.file.seek(SeekFrom::Start(bit_map_offset + HEADER_BYTE_SIZE)));
            let mut byte = try!(self.file.read_u8());
            //println!("[BitSetFile::unset_bit] old byte: {:b}", byte);
            let bit_was_set = byte & mask == mask;
            if bit_was_set{

                byte = byte & !mask;
                //println!("[BitSetFile::unset_bit] new byte: {:b}", byte);

                if self.size() == 1 {
                    try!(self.file.set_len(HEADER_BYTE_SIZE));
                    self.max_bit = 0;
                }
                else if bit == self.max_bit {
                    //println!("[BitSetFile::unset_bit] bit {} is max_bit: {}", bit, self.max_bit);

                    while bit_map_offset > 0 && byte == 0 {
                        bit_map_offset -= 1;
                        let _ = try!(self.file.seek(SeekFrom::Start(bit_map_offset + HEADER_BYTE_SIZE)));
                        byte = try!(self.file.read_u8());
                        //println!("[BitSetFile::unset_bit] LOOP Bit:{} bit_map_offset:{} byte:{:b} ",bit, bit_map_offset, byte);
                    }
                    try!(self.file.set_len(bit_map_offset + 1 + HEADER_BYTE_SIZE)); //+1
                    self.max_bit = max_id_from(bit_map_offset, byte).expect("empty bit_set");
                    //println!("[BitSetFile::unset_bit] new max_bit: {}", self.max_bit);

                }
                else{
                    //println!("[delete] no_shrink: {:b},  bit_map_pos: {}", byte, bit_map_offset);
                    try!(self.file.seek(SeekFrom::Start(bit_map_offset + HEADER_BYTE_SIZE)));
                    try!(self.file.write_u8(byte));
                    }

                self.size -= 1;
                return Ok(true);
            }

        }
        Ok(false)
    }

    pub fn contains_iter(&mut self) -> ContainsIterator {
        ContainsIterator{lower_next: 0, upper_next: self.max_bit, done: self.is_empty(), bsf: self}
    }

    fn read_header(&mut self) -> Result<(u64,u64)> {
        let _ = try!(self.file.seek(SeekFrom::Start(0)));
        let size = try!(self.file.read_u64::<NativeEndian>());
        let max_bit = try!(self.file.read_u64::<NativeEndian>());
        Ok((size, max_bit))
    }

    fn write_header(&mut self) -> Result<()>{
        let _ = try!(self.file.seek(SeekFrom::Start(0)));
        let _ = try!(self.file.write_u64::<NativeEndian>(self.size));
        let _ = try!(self.file.write_u64::<NativeEndian>(self.max_bit));
        Ok(())
    }
}

impl Drop for BitSetFile{
    fn drop(&mut self) {
        let _ = self.file.sync_data();
        self.write_header().expect("Could not write header file");
    }
}

pub struct ContainsIterator<'a> {
    lower_next: u64, // TODO: investigate how to use RangeInclusive (...) for this.
    upper_next: u64,
    done: bool,
    bsf: &'a mut BitSetFile,
}

impl<'a> Iterator for ContainsIterator<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done{
            return None
        }
        while self.lower_next <= self.upper_next{
            let cur_bit = self.lower_next;
            self.lower_next += 1;
            if self.bsf.contains(cur_bit).expect("can not read bit") {
                return Some(cur_bit);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>){
        (0, Some(self.bsf.size() as usize))
    }

}

impl<'a> DoubleEndedIterator for ContainsIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done{
            return None
        }
        while self.lower_next <= self.upper_next{
            let cur_bit = self.upper_next;
            if cur_bit == 0 {
                self.done = true;
            }
            else {
                self.upper_next -= 1;
            }
            if self.bsf.contains(cur_bit).expect("can not use BlockFileContainer") {
                return Some(cur_bit);
            }
        }
        None
    }
}

impl<'a> ExactSizeIterator for ContainsIterator<'a> {
    fn len(&self) -> usize {
        return self.bsf.size() as usize
    }
}

#[test]
fn bit_set_file_new(){
    use std::path::Path;
    use std::fs::{OpenOptions, remove_file};

    let path = Path::new("./test_output/bitsetfile_new.test");
    let file = OpenOptions::new().read(true).write(true).create(true).open(&path).expect("Test file not created");
    let bit_set_file = BitSetFile::new(file);
    assert!(bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 0);
    assert_eq!(bit_set_file.max_bit(), None);
    assert_eq!(bit_set_file.max_bit, 0);
    assert_eq!(bit_set_file.size, 0);
    let metadata = bit_set_file.file.metadata().expect("No metadata for test file");
    assert_eq!(metadata.len(), HEADER_BYTE_SIZE);
    remove_file(&path).unwrap();
}

#[test]
fn bit_set_file_insert(){
    use std::path::Path;
    use std::fs::{OpenOptions, remove_file};

    let path = Path::new("./test_output/bitsetfile_set_bit.test");
    let file = OpenOptions::new().read(true).write(true).create(true).open(&path).expect("Test file not created");
    let mut bit_set_file = BitSetFile::new(file);
    assert!(bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 0);
    assert_eq!(bit_set_file.max_bit(), None);


    let was_not_set = bit_set_file.insert(1).unwrap();
    assert_eq!(was_not_set, true);
    assert!(!bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 1);
    assert_eq!(bit_set_file.max_bit(), Some(1));


    let was_not_set = bit_set_file.insert(0).unwrap();
    assert_eq!(was_not_set, true);
    assert!(!bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 2);
    assert_eq!(bit_set_file.max_bit(), Some(1));


    let was_not_set = bit_set_file.insert(16).unwrap();
    assert_eq!(was_not_set, true);
    assert!(!bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 3);
    assert_eq!(bit_set_file.max_bit(), Some(16));

    let was_not_set = bit_set_file.insert(16).unwrap();
    assert_eq!(was_not_set, false);
    assert!(!bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 3);
    assert_eq!(bit_set_file.max_bit(), Some(16));

    assert_eq!(bit_set_file.max_bit, 16);
    assert_eq!(bit_set_file.size, 3);
    let metadata = bit_set_file.file.metadata().expect("No metadata for test file");
    assert_eq!(metadata.len(), 3 + HEADER_BYTE_SIZE);
    remove_file(&path).unwrap();
}

#[test]
fn bit_set_file_remove(){
    use std::path::Path;
    use std::fs::{OpenOptions, remove_file};

    let path = Path::new("./test_output/bitsetfile_unset_bit.test");
    let file = OpenOptions::new().read(true).write(true).create(true).open(&path).expect("Test file not created");
    let mut bit_set_file = BitSetFile::new(file);
    assert!(bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 0);
    assert_eq!(bit_set_file.max_bit(), None);


    let was_set = bit_set_file.remove(1).unwrap();
    assert_eq!(was_set, false);
    assert!(bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 0);
    assert_eq!(bit_set_file.max_bit(),None);


    let was_not_set = bit_set_file.insert(32).unwrap();
    assert_eq!(was_not_set, true);

    let was_set = bit_set_file.remove(32).unwrap();

    assert_eq!(was_set, true);
    assert!(bit_set_file.is_empty());
    assert_eq!(bit_set_file.size(), 0);
    assert_eq!(bit_set_file.max_bit(), None);

    assert_eq!(bit_set_file.max_bit, 0);
    assert_eq!(bit_set_file.size, 0);
    let metadata = bit_set_file.file.metadata().expect("No metadata for test file");
    assert_eq!(metadata.len(), HEADER_BYTE_SIZE);
    remove_file(&path).unwrap();
}
