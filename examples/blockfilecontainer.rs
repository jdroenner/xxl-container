use std::path::Path;

extern crate xxl_container as xxl;
use xxl::io::blockfilecontainer::BlockFileContainer;
use xxl::container::{Container};

fn main() {
    // The path + prefix of the new blockfilecontainers files
    let path = Path::new("./bfc");

    // Create a new BlockFileContainer from a prefix and with block_size = 16. This should match your HDDs block size: usually this shlould be 512 or 4K bytes
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(path, 16).unwrap();

    // get the block_size for later use
    let block_size = bfc.block_size();

    // create new Blocks Vec<u8> and insert them into the container
    for i in 0 ..255 {
        let block = vec![i; block_size];
        println!("Inserting: {:?}", block);
        let id = bfc.insert(block).unwrap();
        println!("ID of the new entry: {}", id);
    }

    // get over the used/reserved  IDs
    let ids: Vec<u64> = bfc.ids().collect();
    println!("IDs used by the container: {:?}", ids);

    // remove the first 20 entries
    for i in 0 .. 20 {
        let id = ids[i];
        let e = bfc.remove(id);
        println!("Entry for ID: {} contained: {:?}",id ,e);
    }

    // get over the used/reserved IDs
    let ids: Vec<u64> = bfc.ids().collect();
    println!("IDs used by the container: {:?}", ids);

    // reserve some new entries. This will use the free entries first
    for _ in 0..30 {
        let id = bfc.reserve().unwrap();
        println!("New reserved ID: {}", id);
    }

    // get the used/reserved  IDs
    let ids: Vec<u64> = bfc.ids().collect();
    println!("IDs used by the container: {:?}", ids);

    // clear the container
    let _ = bfc.clear();
}
