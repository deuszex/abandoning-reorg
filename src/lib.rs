//! Tree module that only preserves nodes to predetermined depth.
//! This tree always chooses the branch with the longest available "lineage".
//! When a new node is inserted and the current root is "too old", the root
//! is replaced with its child that leads to the longest branch, while
//! all other children are abandoned and removed.
//! Only dependency is std to try to minimize the dependency hell that
//! plagues seemingly every project.

use std::cmp::Eq;
use std::collections::HashMap;
use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::Copy;

#[derive(Clone)]
/// Internal node that serves as a "tree node".
pub struct ReorgNode<K, M> {
    /// key of the node. It is used as its key or name.
    key: K,
    /// Index of the node in the system.
    height: u64,
    /// Value of the node.
    value: u64,
    /// key of the node that is parent to this one.
    parent: K,
    /// All nodes that has this node as their "parent",
    children: Vec<K>,
    /// Custom designated meta data
    custom_meta: M,
}

impl<K: Debug, M: Debug> Display for ReorgNode<K, M> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            ">Key: {:?}\n>Height: {}\n>Value: {}\n>Parent: {:?}\n>Children: {:?}\n>Custom Meta: {:?}",
            self.key, self.height, self.value, self.parent, self.children, self.custom_meta
        )
    }
}

impl<K, M> ReorgNode<K, M> {
    pub fn new(key: K, height: u64, value: u64, parent: K, custom_meta: M) -> ReorgNode<K, M> {
        ReorgNode {
            key,
            height,
            value,
            parent,
            children: Vec::new(),
            custom_meta,
        }
    }
}

impl<K: Default, M: Default> Default for ReorgNode<K, M> {
    fn default() -> Self {
        ReorgNode::new(K::default(), 0, 0, K::default(), M::default())
    }
}

/// Main working struct of the reogranizational code body.
pub struct Organizer<K, M> {
    /// The current root, or oldest node that we deal with.
    root: ReorgNode<K, M>,
    /// Every node currently held in the system, stored by their key as its key.
    /// Does not contain the root.
    nodes_by_key: HashMap<K, ReorgNode<K, M>>,
    /// Every node currently held by the system, stored by their height as the key.
    /// As the main functionality is to decide which branch is the longest, this
    /// map has a Vec as the value field, because multiple nodes with the same
    /// height are possible.
    /// This map does contain the root.
    nodes_by_height: HashMap<u64, Vec<K>>,
    /// Buffer for node that doesn't have their parent in the system yet.
    /// This might be because the nodes height is greater by multiple steps
    /// than the one we currently have as head.
    buffer: HashMap<K, ReorgNode<K, M>>,
    /// The height of the node with currently greatest height in the system.
    /// (Can also be described as the youngest or newest nodes height.)
    /// (Does not include nodes in the buffer)
    height: u64,
    /// The predetermined depth we check the branches to. Any node older than this
    /// are discarded.
    allowed_depth: u64,
}

impl<K: Debug, M: Debug> Display for Organizer<K, M> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Root: \n{}\nNode Key Count: {}\nNode Height Count: {}\nHeight: {:?}\nAllowed Depth: {:?}", 
        self.root, self.nodes_by_key.len(), self.nodes_by_height.len(), self.height, self.allowed_depth)
    }
}

impl<K: Default, M: Default> Default for Organizer<K, M> {
    fn default() -> Self {
        Organizer {
            height: 0,
            root: ReorgNode::default(),
            nodes_by_key: HashMap::new(),
            nodes_by_height: HashMap::new(),
            buffer: HashMap::new(),
            allowed_depth: 255,
        }
    }
}

impl<K: Default + Eq + Hash + Clone + Debug + Copy, M: Debug + Default> Organizer<K, M> {
    /// Default state constructor with predetermined max depth.
    pub fn new(allowed_depth: u64) -> Organizer<K, M> {
        Self {
            height: 0,
            root: ReorgNode::default(),
            nodes_by_key: HashMap::new(),
            nodes_by_height: HashMap::new(),
            buffer: HashMap::new(),
            allowed_depth,
        }
    }

    /// Constructor function that takes the first root node - possibly the genesis node -
    /// and the depth we want to allow reorganization to. Stores the root node in its slot,
    /// as well as by height. Root is not stored by key, only by height.
    pub fn new_with(root: ReorgNode<K, M>, allowed_depth: u64) -> Organizer<K, M> {
        let mut nodes_by_height = HashMap::new();
        nodes_by_height.insert(root.height, vec![root.key]);
        Self {
            height: root.height,
            root,
            nodes_by_key: HashMap::new(),
            nodes_by_height,
            buffer: HashMap::new(),
            allowed_depth,
        }
    }

    /// Init function, sets a new root.
    pub fn init(&mut self, first_root: ReorgNode<K, M>) {
        self.height = first_root.height;
        self.nodes_by_height
            .insert(first_root.height, vec![first_root.key]);
        self.root = first_root;
    }

    /// Returns the difference of height and the allowed depth to determine the highest node
    /// that we allow, or zero if the number would be negative.
    fn allowed_oldest(&self) -> u64 {
        self.height.saturating_sub(self.allowed_depth as u64)
    }

    /// This function is part of the garbage collection. Deletes every node that in the branch
    /// stemming from the node we designated.
    pub fn delete_children(&mut self, branch_root: &K) -> Vec<ReorgNode<K, M>> {
        let mut ret: Vec<ReorgNode<K, M>> = Vec::new();
        // First we try to remove the designated node from the system
        if let Some(removed) = self.nodes_by_key.remove(branch_root) {
            // We add the removed nodes children to the list that we will remove next
            let mut removeable: Vec<K> = removed.children.clone();
            // We push the node into the list of nodes we will return
            ret.push(removed);
            // As long as there are possible nodes in this branch we repeatedly
            // remove a node, if it succeeds we push its children to
            // the list of removable nodes, then append the node to the return list.
            while !removeable.is_empty() {
                let mut remove_next = Vec::new();
                for key in &removeable {
                    if let Some(mut removed_last) = self.nodes_by_key.remove(key) {
                        remove_next.append(&mut removed_last.children);
                        ret.push(removed_last);
                    }
                }
                removeable = remove_next;
            }
        }
        ret
    }

    /// Utility function that lists node stored by their keyes. (Only prints the keyes)
    pub fn list_node_keyes(&self) {
        for key in self.nodes_by_key.keys() {
            println!("{:?}", key)
        }
    }

    /// Utility that prints the node stored by their keyes. (Actually displays the nodes)
    pub fn list_node(&self) {
        for node in self.nodes_by_key.values() {
            println!("{}\n", node)
        }
    }

    /// Returns the key of the node that is the immidiate child of the current root,
    /// and has the longest available lineage.
    /// # Panics
    /// If this function call fails that means that at least one node was not stored in the memory.
    fn find_longest_branch(&self) -> K {
        // We take the nodes that correspond to the greatest available
        // height stored in the system as the heads of the tree.
        // This should not fail for we always store every node by their height.
        let heads = self
            .nodes_by_height
            .get(&self.height)
            .expect("there in no node stored corresponding to the greatest logged height");
        let mut lead_branches: HashMap<K, u64> = HashMap::new();
        // We check each head of the tree
        for head in heads {
            let mut height = 0;
            let mut root = head;
            // We count the lineage number of each branch from head to root
            while let Some(node) = self.nodes_by_key.get(root) {
                if node.parent != self.root.key {
                    root = &node.parent;
                    height += 1;
                } else {
                    // When we reached the roots immidiate child we break out of the loop
                    break;
                }
            }
            // Insert the height of the branch with the branches root (the system roots child)
            // as the key.
            lead_branches.insert(*root, height);
        }
        let (mut longest_key, mut longest_height) = (K::default(), 0);
        // After the parsed every branch corresponding to a head, we determine the longest and return
        // its key
        for (key, height) in lead_branches {
            if height > longest_height {
                longest_height = height;
                longest_key = key;
            }
        }
        longest_key
    }

    /// Apply callback from given head to given root, or as long as possible.
    pub fn apply_callback<T>(
        &self,
        head: K,
        root: Option<K>,
        callback: &mut dyn FnMut(&ReorgNode<K, M>) -> T,
    ) {
        let head_node = self
            .nodes_by_key
            .get(&head)
            .expect("there in no node stored corresponding to the gived key");
        callback(head_node);
        let mut cursor = head_node.parent;
        while let Some(node) = self.nodes_by_key.get(&cursor) {
            match root {
                Some(root_key) => {
                    if node.key != root_key {
                        cursor = node.parent
                    } else {
                        break;
                    }
                }
                None => {
                    cursor = node.parent;
                }
            }
            callback(node);
        }
    }

    /// Utility function that takes the lists of nodes stored by key and nodes stored
    /// by their height, and checks for node that are only logged by height and not by key.
    /// This should always only return the current root.
    pub fn check_height_to_key_diff(&self) -> Vec<K> {
        let mut ret = HashMap::new();
        for nodes in self.nodes_by_height.values() {
            for b in nodes {
                ret.insert(*b, ());
            }
        }
        for key in self.nodes_by_key.keys() {
            ret.remove(key);
        }
        ret.keys().copied().collect::<Vec<K>>()
    }

    /// Main logic of the reorganizational functionality. Determines the validity of the
    /// inserted node by checking its height and its parent then
    /// saves it into a branch if a viable parent is present and the height is acceptable,
    /// or into the buffer if parent is not present but has a good height.
    /// Otherwise the node is discarded.
    /// The height of the node is considered good if its greater than that of the current root.
    /// Panics
    /// A panic will occur if a node has a child listed that we do not have
    /// stored by its key.
    pub fn insert(&mut self, node: ReorgNode<K, M>) {
        // if new node older than we search, we don't care about it
        if node.height <= self.allowed_oldest() {
            return;
        }
        // if new nodes parent isn't stored already and it's height isn't greater than
        // what we know the newest to be, we don't care about it
        if !self.nodes_by_key.contains_key(&node.parent) && node.height <= self.height {
            return;
        }
        // when the root nodes depth reaches the threshold we predetermined
        if self.root.height == self.allowed_oldest() {
            match self.root.children.len() {
                0 => {}
                1 => {
                    // In case the root has only one child, the child becomes the new node.
                    // If this fails that means the children of the root were already removed.
                    self.root = self.nodes_by_key.remove(&self.root.children[0]).unwrap();
                }
                _ => {
                    // In case the root has multiple children we determine the longest branch.
                    let remove = self.root.children.clone();
                    // We replace the current root with its child that heirs the longest lineage.
                    // If this fails that means that the branch has already been removed.
                    self.root = self
                        .nodes_by_key
                        .remove(&self.find_longest_branch())
                        .unwrap();
                    for dead_branch in remove {
                        // we delete every branch stemming from the root other than the longest one
                        if dead_branch != self.root.key {
                            self.delete_children(&dead_branch);
                        }
                    }
                }
            }
        }
        // Retrieving the inserted nodes parent to append said node to the
        // parents list of children. If neither ifs trigger than parent is not part
        // of the system, and we put the node into the buffer.
        if let Some(parent) = self.nodes_by_key.get_mut(&node.parent) {
            parent.children.push(node.key);
        } else if node.parent == self.root.key {
            self.root.children.push(node.key);
        } else {
            self.buffer.insert(node.key, node);
            return;
        }
        // We save the node key to its height
        match self.nodes_by_height.get_mut(&node.height) {
            Some(has_node) => has_node.push(node.key),
            None => {
                self.nodes_by_height.insert(node.height, vec![node.key]);
            }
        };
        // This is the newest node hence we take its height as the new system height
        self.height = node.height;
        // We save the node itself with its key as the key
        self.nodes_by_key.insert(node.key, node);
        // We double check for nodes that should have already been removed
        self.nodes_by_key.remove(&self.root.parent);
        if let Some(old_root) = self
            .nodes_by_height
            .remove(&(self.root.height.saturating_sub(1)))
        {
            for old in old_root {
                self.nodes_by_key.remove(&old);
            }
        }

        let mut reinsert = Vec::new();
        let mut buffer_clear = Vec::new();
        // We check the nodes in the buffer wether they have expired,
        // then we check if their parents have been pushed into the system.
        for (key, buffer_node) in &self.buffer {
            if buffer_node.height < self.allowed_oldest() {
                buffer_clear.push(*key);
                continue;
            }
            if let Some(parent) = self.nodes_by_key.get_mut(&buffer_node.parent) {
                parent.children.push(*key);
                reinsert.push(*key);
            }
        }
        // If we found the parent of a node in the buffer, we save it
        for r in reinsert {
            if let Some(reinsertable) = self.buffer.remove(&r) {
                match self.nodes_by_height.get_mut(&reinsertable.height) {
                    Some(has_node) => has_node.push(r),
                    None => {
                        self.nodes_by_height.insert(reinsertable.height, vec![r]);
                    }
                };
                self.nodes_by_key.insert(r, reinsertable);
            }
        }
        // If the node has expired we remove if from the buffer-
        for bc in buffer_clear {
            self.buffer.remove(&bc);
        }
        // TODO handle nodes with greater height than current height
        // TODO handle nodes in the acceptable height that doesn't have
        //      parent in the system.
        // buffer?
    }

    pub fn highest_nodes(&self) -> &[K] {
        self.nodes_by_height.get(&self.height).unwrap()
    }
}

/// Utility function that creates a key([u8;32]) from a u64
fn utoa(u: u64) -> [u8; 32] {
    let mut ret = [0u8; 32];
    let mut vec = u.to_ne_bytes().to_vec();
    vec.append(&mut [0u8; 24].to_vec());
    ret.copy_from_slice(&vec);
    ret
}

/// Test callback function
fn callback(node: &ReorgNode<[u8; 32], ()>) {
    println!("{:?} : {}", node.key, node.height);
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
    cb.apply_callback(utoa(3009), Some(utoa(3000)), &mut callback);
    // cb.list_nodes();
    // println!("deleting branch");
    // cb.delete_children(utoa(2));
    // cb.list_nodes();
    assert!(false)
}
