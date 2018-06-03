//! Stack of widget IDs.
use std::hash::{Hash, Hasher, SipHasher};

/// The ID type.
pub type ItemID = u64;

/// The ID stack. Each level corresponds to a parent ItemNode.
pub struct IdStack(pub(super) Vec<ItemID>);

impl IdStack {
    /// Creates a new IdStack and push the specified ID onto it.
    pub fn new(root_id: ItemID) -> IdStack {
        IdStack(vec![root_id])
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> ItemID {
        let stacklen = self.0.len();
        let key1 = if stacklen >= 2 {
            self.0[stacklen - 2]
        } else {
            0
        };
        let key0 = if stacklen >= 1 {
            self.0[stacklen - 1]
        } else {
            0
        };
        let mut sip = SipHasher::new_with_keys(key0, key1);
        s.hash(&mut sip);
        sip.finish()
    }

    /// Hashes the given data, initializing the hasher with the items currently on the stack.
    /// Pushes the result on the stack and returns it.
    /// This is used to generate a unique ID per item path in the hierarchy.
    pub fn push_id<H: Hash>(&mut self, s: &H) -> ItemID {
        let id = self.chain_hash(s);
        let parent_id = *self.0.last().unwrap();
        self.0.push(id);
        id
    }

    /// Pops the ID at the top of the stack.
    pub fn pop_id(&mut self) {
        self.0.pop();
    }
}
