extern crate xxl_container as xxl;
use xxl::container::{Container, CloneContainer};
use xxl::mem::veccontainer::VecContainer;

fn main() {

    // Create a new VecContainer
    let mut vec_c = VecContainer::<f64>::with_capacity(10);
    println!("This is how a VecContainer looks after ceation: {:#?}", vec_c);

    // inserting an element returns the corresponding id
    let one = vec_c.insert(1.1).unwrap();
    println!("Inserting 1.1 returns the ID {}", one);
    let two = vec_c.insert(2.2).unwrap();
    println!("Inserting 2.2 returns the ID {}", two);

    // print some informations
    println!("After the inserts the Container contains two entrys: {:?}", vec_c);

    // removing an id returns the element stored at this id.
    let element = vec_c.remove(one).unwrap();
    println!("Using remove with ID {} returns {:?}", one, element);
    println!("After removing ID {} there is a free slot. Also the ID was moved to the free_list: {:?}", one, vec_c);

    // reserving an element returns an ID without inserting an element into the container
    let reserved_id = vec_c.reserve().unwrap();
    println!("Reserving a slot returns a new ID: {}", reserved_id);
    println!("After reserving the container has a reserved slot. The ID ({}) was removed from the free_list: {:?}",  reserved_id, vec_c);

    // updating an entry returns Some(old_entry) if the slot was used or None if the slot was only reserved.
    let old_entry = vec_c.update(reserved_id, 12.12).unwrap();
    println!("Update returns the old entry for an ID. For {} this was {:?}.", reserved_id, old_entry);
    println!("The Container now contains the updated element for ID {}: {:?}", reserved_id, vec_c);

    // VecContainer also implements CloneContainer for elemnts which implement clone.
    let new_entry_clone = vec_c.get_clone(reserved_id).unwrap();
    println!("Elements can also be cloned. The clone of ID {} is {:?}.", reserved_id, new_entry_clone);
    println!("This does not change the container: {:?}", vec_c);
}
