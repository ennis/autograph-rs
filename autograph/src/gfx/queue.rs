use super::fence::{Fence, FenceValue};
use super::context::{Context, ContextConfig};
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::collections::VecDeque;

use std::marker::PhantomData;

use super::buffer::*;
use super::texture::*;

// A frame: instances are alive until the frame is complete
pub struct Frame<'queue> {
    // Associated queue
    queue: &'queue FrameQueue,
    // Resources held onto by this frame
    pub(super) ref_buffers: RefCell<Vec<Arc<BufferAny>>>,
    pub(super) ref_textures: RefCell<Vec<Arc<Texture>>>,
}

impl<'queue> Frame<'queue> {
    // consume self, also releases the borrow on queue
    pub fn submit(self) {
        //debug!("Submit frame: sync index={:?}", self.queue.fence.borrow().next_value());
        // setup fence in command stream
        let sync = self.queue.fence.borrow_mut().advance_async();
        // release lock
        self.queue.has_live_frames.set(false);
        // add ourselves to the list of live frames
        self.queue.submitted_frames.borrow_mut().push_back(
            SubmittedFrame {
                sync,
                ref_buffers: self.ref_buffers.into_inner(),
                ref_textures: self.ref_textures.into_inner(),
            },
        );
    }
}

struct SubmittedFrame {
    sync: FenceValue,
    ref_buffers: Vec<Arc<BufferAny>>,
    ref_textures: Vec<Arc<Texture>>,
}

// Represents a succession of frames
pub struct FrameQueue {
    ctx: Arc<Context>,
    fence: RefCell<Fence>,
    has_live_frames: Cell<bool>,
    // submitted but not completed frames, hold refs to resources
    submitted_frames: RefCell<VecDeque<SubmittedFrame>>,
}

impl<'queue> Frame<'queue> {
    pub fn fence_value(&self) -> FenceValue {
        self.queue.fence.borrow().next_value()
    }

    pub fn queue<'a>(&'a self) -> &'a FrameQueue {
        self.queue
    }
}

impl FrameQueue {
    pub fn new(ctx: &Arc<Context>) -> FrameQueue {
        FrameQueue {
            ctx: ctx.clone(),
            fence: RefCell::new(Fence::new(ctx.clone(), FenceValue(-1))),
            has_live_frames: Cell::new(false),
            submitted_frames: RefCell::new(VecDeque::new()),
        }
    }

    pub fn context(&self) -> &Arc<Context> {
        &self.ctx
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
    // 1. Use a Arc<RefCell<Queue>>, but well...
    // 2. Dynamically check that there is only one frame
    // 3. Have UploadBuffer not borrow the queue on creation => not good, since we
    // can't check that we use the right queue on each call to UploadBuffer::allocate()
    pub fn new_frame<'a>(&'a self) -> Frame<'a> {
        if self.has_live_frames.get() {
            panic!(
                "Cannot have two simultaneous Frame objects alive. Drop the current Frame object before calling new_frame."
            );
        }

        let current_sync = self.fence.borrow_mut().current_value();

        // collect frames that are done
        let mut submitted_frames = self.submitted_frames.borrow_mut();
        submitted_frames.retain(|frame| frame.sync > current_sync);
        //debug!("Number of live submitted frames: {}", submitted_frames.len());

        Frame {
            queue: &self,
            ref_textures: RefCell::new(Vec::new()),
            ref_buffers: RefCell::new(Vec::new()),
        }
    }
}

pub fn create_context_and_frame_queue(config: &ContextConfig) -> (Arc<Context>, FrameQueue) {
    unimplemented!()
}
