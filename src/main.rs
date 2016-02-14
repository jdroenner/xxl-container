//use std::vec::Vec;

use std::path::Path;

extern crate xxl_container as xxl;
use xxl::io::blockfilecontainer::BlockFileContainer;
//use xxl::mem::veccontainer::VecContainer;
use xxl::container::{Container};
use xxl::adapter::serde::{SerdeAdapter};

fn main() {
    let path = Path::new("c:/temp/bfc");

    println!("{:?}", path);
    let mut bfc = BlockFileContainer::new_from_prefix_and_block_size(path, 16 as usize).unwrap();

    //for j in 2u8 .. 4{
        for i in 0 ..255 {
            let e = bfc.insert(vec!(i));
            println!("_________________________EEE ID: {} RESULT:{:?}",i ,e);
        }

        for i in bfc.used_ids() {
            println!("I {:?}", i);
        }

        for i in 0..200 {
            let e = bfc.remove(i*2 as u64);
            println!("_________________________FFF ID: {} RESULT:{:?}",i ,e);
        }

        for i in 0..200 {
            let e = bfc.reserve();
            println!("_________________________FFF ID: {} RESULT:{:?}",i ,e);
        }

        for i in bfc.used_ids() {
            println!("U {:?}", i);
        }

        for i in bfc.reserved_ids().rev() {
            println!("R {:?}", i);
        }

        println!("Size {:?}", bfc.size());

        let _ = bfc.clear();



        //let mut edc = EncoderDecoderContainer::new(|x: u64| vec!((x as u8)), |y: Vec<u8>| (y[0] as u64), bfc );
        //let res = edc.insert(11);
        //let res = edc.insert(22);
        //let res = edc.insert(33);
        //let res = edc.insert(44);
        //println!("{:?}", res);

        let _ = bfc.clear();

        let mut scc = SerdeAdapter::new(bfc);
        let _ = scc.insert(String::from("MEH")).unwrap();
        let _ = scc.insert(String::from("wuff")).unwrap();
        let _ = scc.insert(String::from("hurz")).unwrap();
        let id = scc.insert(String::from("gna")).unwrap();
        println!("{:?}", id);
        let res = scc.remove(id).unwrap();
        println!("{:?}", res);


    //}

/*
    let id = bfc.insert(Vec::from("11111111"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("22222222"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("33333333"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("44444444"));
    println!("ID: {:?}", id);

    let meh = bfc.remove(0);
    println!("meh: {:?}", meh);
    let meh = bfc.remove(1);
    println!("meh: {:?}", meh);
    let meh = bfc.remove(2);
    println!("meh: {:?}", meh);
    let meh = bfc.remove(3);
    println!("meh: {:?}", meh);

    let id = bfc.insert(Vec::from("55555555"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("66666666"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("77777777"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("88888888"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("99999999"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("00000000"));
    println!("ID: {:?}", id);
    let meh = bfc.remove(3);
    println!("meh: {:?}", meh);
    let meh = bfc.remove(5);
    println!("meh: {:?}", meh);

    let id = bfc.insert(Vec::from("11111111"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("22222222"));
    println!("ID: {:?}", id);
    let id = bfc.insert(Vec::from("33333333"));
    println!("ID: {:?}", id);

    println!("{:?}",bfc);
    {
        println!("check");
        let entry = bfc.entry(1);
        //println!("Vacant: {:?}", entry);
        if let Entry::Vacant(vacant_entry) = entry {
            println!("vac");
            let _ = vacant_entry.insert(Vec::from("|VA INS|"));
        //    print!("{:?}", string);
        }
    }
    println!("CHECK");
    {
        let entry = bfc.entry(1);
        //println!("{:?}", entry);
        if let Entry::Occupied(mut oc_entry) = entry {
            println!("OC");
            {
                let block = oc_entry.get();
                println!("Oc: {:?}", String::from_utf8_lossy(block));
            }
            let old = oc_entry.update(Vec::from("|OCtabla INS|"));
            println!("oc old: {:?}", old);
        }
    }

    for i in 0u8..16 {
        let _ = bfc.insert(vec!(i));
    }

    for i in 0..17 {
        if let Entry::Occupied(mut e) = bfc.entry(i){
            let old = e.update(Vec::from(&*format!("{}", i*17)));
            println!("{} -> {:?}", i, old);
        }
    }



    let mut vec_c = VecContainer::<f64>::with_capacity(10);
    let one = vec_c.insert(1.1);
    let two = vec_c.insert(2.2);
    println!("{:?}, {:?}", one, two);
    let _ = vec_c.remove(one);
    println!("{:?}", vec_c);
    let _ = vec_c.insert(3.3);
    let _ = vec_c.insert(4.4);
    println!("{:?}", vec_c);
    let five = vec_c.insert(5.5);
    let _ = vec_c.remove(two);
    println!("{:?}", vec_c);
    let _ = vec_c.insert(6.6);
    println!("{:?}", vec_c);
    vec_c.remove(0);
    vec_c.remove(1);
    vec_c.remove(2);
    vec_c.remove(4);
    vec_c.remove(5);
    println!("{:?}", vec_c);
    {
        let five_r = vec_c.get(five);
        println!("five: {:?} -> {:?}", five, five_r);
    }
    println!("{:?}", vec_c);
    vec_c.remove(five);
    println!("{:?}", vec_c);
    */

}
