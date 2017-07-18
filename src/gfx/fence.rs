use gl;
use gl::types::*;
use libc::c_void;
use std::marker::PhantomData;
use std::mem;
use std::collections::vec_deque::VecDeque;
use super::context::Context;
use std::rc::Rc;

struct SyncPoint
{
    sync: GLsync,
    target: i64
}

///
///
pub struct Fence
{
    sync_points: VecDeque<SyncPoint>,
    current_value: i64,
}

impl Fence
{
    pub fn new(ctx: Rc<Context>, init_value: i64) -> Fence {
        Fence {
            sync_points: VecDeque::new(),
            current_value: init_value
        }
    }

    pub fn signal(&self) {
        unimplemented!()
    }

    pub fn advance(&mut self) {
        unimplemented!()
    }

    pub fn wait(&self) {
        unimplemented!()
    }

    pub fn get_value(&self) -> i64 {
        unimplemented!()
    }
}