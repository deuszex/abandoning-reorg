use abandoning_reorg::{
    ReorgNode,
    Organizer
};

/// Utility function that creates a key([u8;32]) from a u64
fn utoa(u: u64) -> [u8; 32] {
    let mut ret = [0u8; 32];
    let mut vec = u.to_ne_bytes().to_vec();
    vec.append(&mut [0u8; 24].to_vec());
    ret.copy_from_slice(&vec);
    ret
}

fn create_test_filled() -> Organizer<[u8;32], ()>{
    let genesis = ReorgNode::new(utoa(0), 0, 0, utoa(999999999), ());
    let mut cb = Organizer::new(255);
    cb.init(genesis);
    for i in 1..2000 {
        cb.insert(ReorgNode::new(utoa(i), i, 0, utoa(i - 1), ()))
    }
    cb
}

/// Test callback function
fn callback(node: &ReorgNode<[u8; 32], ()>) {
    println!("{:?} : {}", node.key(), node.height());
}

#[test]
fn new_test() {
    Organizer::<[u8;32], ()>::new(255);
}

#[test]
fn default_test() {
    Organizer::<[u8;32], ()>::default();
}

#[test]
fn new_with_test() {
    Organizer::<[u8;32], ()>::new_with(ReorgNode::default(), 255);
}

#[test]
fn insert_test() {
    let genesis = ReorgNode::new(utoa(0), 0, 0, utoa(999999999), ());
    println!("genesis: \n{}", genesis);
    let mut cb = Organizer::new(255);
    println!("\npreinit state \n{}", cb);
    cb.init(genesis);
    println!("\npost init state \n{}", cb);
    for i in 1..2000 {
        cb.insert(ReorgNode::new(utoa(i), i, 0, utoa(i - 1), ()))
    }
}

#[test]
fn callback_test() {
    let genesis = ReorgNode::new(utoa(0), 0, 0, utoa(999999999), ());
    println!("genesis: \n{}", genesis);
    let mut cb = Organizer::new(255);
    println!("\npreinit state \n{}", cb);
    cb.init(genesis);
    println!("\npost init state \n{}", cb);
    for i in 1..2000 {
        cb.insert(ReorgNode::new(utoa(i), i, 0, utoa(i - 1), ()))
    }
}

#[test]
fn fail_test() {
    let org = create_test_filled();
    org.apply_callback(Some(utoa(4000)), None, &mut callback);
}

#[test]
fn test() {
    // Test intentionally fails
    let genesis = ReorgNode::new(utoa(0), 0, 0, utoa(999999999), ());
    println!("genesis: \n{}", genesis);
    let mut cb = Organizer::new(255);
    println!("\npreinit state \n{}", cb);
    cb.init(genesis);
    println!("\npost init state \n{}", cb);
    for i in 1..2000 {
        cb.insert(ReorgNode::new(utoa(i), i, 0, utoa(i - 1), ()))
    }
    println!("\ntree before pushing extra branches \n{}", cb);
    for i in 0..10 {
        cb.insert(ReorgNode::new(utoa(2000 + i), 1996, 0, utoa(1995), ()));
    }
    println!("\ntree after pushing extra branches \n{}", cb);
    for i in 0..1000 {
        cb.insert(ReorgNode::new(
            utoa(2010 + i),
            1997 + i,
            0,
            utoa(2009 + i),
            (),
        ));
    }
    println!("\ntree after continuing one of the branches \n{}", cb);
    println!("-----------");
    println!(
        "This should be the same as the root and nothing else\n{:?}",
        cb.check_height_to_key_diff()
    );
    println!("Highest node(s): {:?}", cb.highest_nodes());
    cb.apply_callback(Some(utoa(3009)), Some(utoa(3000)), &mut callback);
    cb.list_nodes();
    println!("deleting branch");
    cb.delete_children(&utoa(2850));
    cb.list_nodes();
    // assert!(false)
}
