use std::fs::{File};
use std::io::{Result, Seek, SeekFrom};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};


#[derive(Debug)]
pub struct StackFile{
    file: File,
    entries: u64,
}

impl StackFile{

    pub fn next(&mut self) -> Result<Option<u64>>{
        if !self.is_empty() {
            let offset = try!(self.file.seek(SeekFrom::End(-8))); //move one u64 from the end
            let element  = try!(self.file.read_u64::<NativeEndian>()); // read one u64
            try!(self.file.set_len(offset));   // "remove" the last u64
            self.entries -= 1;
            Ok(Some(element))
        }
        else{
            Ok(None)
        }
    }
    pub fn add(&mut self, element:u64) -> Result<u64>{
        try!(self.file.seek(SeekFrom::End(0)));
        try!(self.file.write_u64::<NativeEndian>(element));
        self.entries += 1;
        Ok(self.entries)
    }

    pub fn is_empty(&self) -> bool{
        self.entries() == 0
    }

    pub fn entries(&self) -> u64{
        self.entries
    }

    pub fn clear(&mut self) -> Result<()>{
        self.entries = 0;
        self.file.set_len(0)
    }


    pub fn new(file: File) -> Self {
        StackFile{file: file, entries: 0}
    }

    pub fn open(file:File) -> Result<Self> {
        let meta = try!(file.metadata());
        let entries = meta.len() / 8;
        Ok(StackFile{file: file, entries: entries})
    }
}

impl Drop for StackFile{
    fn drop(&mut self) {
        let _ = self.file.sync_data();
    }
}

#[test]
fn stack_file_new(){
    use std::path::Path;
    use std::fs::{OpenOptions, remove_file};

    let path = Path::new("./test_output/stackfile_new.test");
    let file = OpenOptions::new().read(true).write(true).create(true).open(&path).expect("Test file not created");
    let stack_file = StackFile::new(file);
    assert!(stack_file.is_empty());
    assert_eq!(stack_file.entries(), 0);
    assert_eq!(stack_file.entries, 0);
    let metadata = stack_file.file.metadata().expect("No metadata for test file");
    assert_eq!(metadata.len(), 0);
    remove_file(&path).unwrap();
}

#[test]
fn stack_file_add(){
    use std::path::Path;
    use std::fs::{OpenOptions, remove_file};

    let path = Path::new("./test_output/stackfile_add.test");
    let file = OpenOptions::new().read(true).write(true).create(true).open(&path).expect("Test file not created");
    let mut stack_file = StackFile::new(file);
    stack_file.add(8).unwrap();
    assert!(!stack_file.is_empty());
    assert_eq!(stack_file.entries(), 1);
    assert_eq!(stack_file.entries, 1);
    let metadata = stack_file.file.metadata().expect("No metadata for test file");
    assert_eq!(metadata.len(), 8);
    remove_file(&path).unwrap();
}

#[test]
fn stack_file_next(){
    use std::path::Path;
    use std::fs::{OpenOptions, remove_file};

    let path = Path::new("./test_output/stackfile_next.test");
    let file = OpenOptions::new().read(true).write(true).create(true).open(&path).expect("Test file not created");
    let mut stack_file = StackFile::new(file);
    let next = stack_file.next().expect("Could not read next value");
    assert_eq!(next, None);
    stack_file.add(8u64).unwrap();
    assert!(!stack_file.is_empty());
    assert_eq!(stack_file.entries(), 1);
    let next = stack_file.next().expect("Could not read next value");
    assert_eq!(next, Some(8u64));
    assert_eq!(stack_file.entries, 0);
    let metadata = stack_file.file.metadata().expect("No metadata for test file");
    assert_eq!(metadata.len(), 0);
    remove_file(&path).unwrap();
}
