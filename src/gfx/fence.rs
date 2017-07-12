use gl;
use gl::types::*;
use libc::c_void;
use std::marker::PhantomData;
use std::mem;
use std::collections::vec_deque::VecDeque;


struct SyncPoint
{
    sync: GLsync,
    target: i64
}

///
///
struct Fence<'ctx>
{
    sync_points: VecDeque<SyncPoint>,
    current_value: i64,
    _phantom: PhantomData<&'ctx ()>
}

impl<'ctx> Fence<'ctx>
{

}