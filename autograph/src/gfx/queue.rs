use super::fence::{Fence, FenceValue};
use super::context::{Context, ContextConfig};
use std::cell::{RefCell,Cell};
use std::rc::Rc;

use std::marker::PhantomData;

pub struct Frame<'queue> {
    // Associated queue
    queue: &'queue FrameQueue
}

impl<'queue> Frame<'queue>
{
    // consume self, also releases the borrow on queue
    pub fn submit(self) {
        debug!("Submit frame: sync index={:?}", self.queue.fence.borrow().next_value());
        self.queue.fence.borrow_mut().advance_async();
        self.queue.has_live_frames.set(false);
    }
}

// Represents a succession of frames
pub struct FrameQueue
{
    ctx: Rc<Context>,
    fence: RefCell<Fence>,
    has_live_frames: Cell<bool>
}

impl<'queue> Frame<'queue>
{
    pub fn fence_value(&self) -> FenceValue
    {
        self.queue.fence.borrow().next_value()
    }

    pub fn queue<'a>(&'a self) -> &'a FrameQueue {
        self.queue
    }
}

impl FrameQueue
{
    pub fn new(ctx: Rc<Context>) -> FrameQueue {
        FrameQueue {
            ctx: ctx.clone(),
            fence: RefCell::new(Fence::new(ctx.clone(), FenceValue(-1))),
            has_live_frames: Cell::new(false)
        }
    }

    // TODO: return a reference
    pub fn context(&self) -> Rc<Context>
    {
        self.ctx.clone()
    }

    pub fn current_frame_index(&self) -> u64 {
        self.fence.borrow().next_value().0 as u64
    }

    // Returns -1 if no frame is done yet
    // (more practical than returning an Option<FenceValue>)
    pub fn last_completed_frame(&self) -> FenceValue {
        self.fence.borrow_mut().current_value()
    }

    pub fn next_frame_fence_value(&self) -> FenceValue {
        self.fence.borrow().next_value()
    }

    // Note: the mutable borrow here should prevent the user from
    // creating two concurrent frames
    // however, we can't really do that, since we still need access to the queue
    // while a frame is alive (namely, in UploadBuffers, for reclaiming
    // data)
    // Actually, UploadBuffers just need the fence, but it will still borrow the
    // queue
    // Solutions:
    // 1. Use a Rc<RefCell<Queue>>, but well...
    // 2. Dynamically check that there is only one frame
    // 3. Have UploadBuffer not borrow the queue on creation => not good, since we
    // can't check that we use the right queue on each call to UploadBuffer::allocate()
    pub fn new_frame<'a>(&'a self) -> Frame<'a> {
        if self.has_live_frames.get() {
            panic!("Cannot have two simultaneous Frame objects alive. Drop the current Frame object before calling new_frame.");
        }

        Frame {
            queue: &self,
        }
    }
}

pub fn create_context_and_frame_queue(config: &ContextConfig) -> (Rc<Context>, FrameQueue)
{
    unimplemented!()
}