use std::vec::Vec;
use std::collections::HashMap;

#[derive(Copy,Clone,Hash,Debug,Default,PartialEq,Eq)]
pub struct ID
{
    idx: u32,
    gen: u32
}

impl ID
{
    pub fn null() -> ID { ID { idx: 0, gen: 0 } }
}

pub struct IDTable
{
    live_ids: Vec<ID>,
    free_ids: Vec<ID>
}

impl IDTable
{
    pub fn new() -> IDTable {
        IDTable {
            live_ids: Vec::new(),
            free_ids: Vec::new()
        }
    }

    // no need for any interior mutability here since
    // the ID does not borrow anything
    pub fn create_id(&mut self) -> ID {
        if let Some(free_id) = self.free_ids.pop() {
            free_id
        } else {
            let new_id = ID { idx: self.num_live_ids(), gen: 1 };
            self.live_ids.push(new_id);
            new_id
        }
    }

    pub fn delete_id(&mut self, id: ID) {
        // increase generation
        self.live_ids[id.idx as usize].gen += 1;
        // add to free list
        self.free_ids.push(self.live_ids[id.idx as usize]);
    }

    pub fn num_live_ids(&self) -> u32 {
        self.live_ids.len() as u32
    }

    pub fn is_valid(&self, id: ID) -> bool {
        id.idx < self.num_live_ids() && id.gen < self.live_ids[id.idx as usize].gen
    }

    pub fn collect<T>(&self, map: &mut HashMap<ID, T>) {
        map.retain(|&id,_| self.is_valid(id) )
    }
}

// scene_loader:
// id_table
// all component hashes
// resource cache
// resources should be cached

