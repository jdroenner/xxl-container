use std::path::Path;

extern crate xxl_container as xxl;
use xxl::io::blockfilecontainer::BlockFileContainer;
use xxl::container::{Container};
use xxl::adapter::serde::{SerdeAdapter};

fn main() {

        // The path + prefix of the new blockfilecontainers files
        let path = Path::new("./bfc");

        // Create a new BlockFileContainer from a prefix and with block_size = 16. This should match your HDDs block size: usually this shlould be 512 or 4K bytes
        let bfc = BlockFileContainer::new_from_prefix_and_block_size(path, 16).unwrap();

        // Create a new convertig container by wrapping the bfc with a SerdeAdapter
        let mut scc = SerdeAdapter::new(bfc);

        // insert some words into the container
        for word in "A few words".split_whitespace() {
            let id = scc.insert(String::from(word)).unwrap();
            println!("Inserted: {:?} -> ID = {}", word, id);
        }

        // reserve some additional ids
        for _ in 0..10 {
            scc.reserve().unwrap();
        }

        // get the used/reserved IDs
        let ids: Vec<u64> = scc.ids().collect();
        println!("IDs used by the container: {:?}", ids);

        // remove the entries for each id
        for id in ids{
            let entry = scc.remove(id).unwrap();
            println!("Entry for {} -> {:?}", id, entry);
        }

        // clear the container/adapter
        let _ = scc.clear();
    }
