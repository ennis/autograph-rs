use super::queue::{Queue, FrameResources};
use super::buffer_data::BufferData;
use super::sampler::SamplerDesc;
use super::texture::TextureAny;
use super::upload_buffer::UploadBuffer;
use super::buffer::{BufferSlice,RawBuffer,RawBufferSlice};
use super::framebuffer::{Framebuffer, FramebufferObject};
use super::bind::{VertexInput, Uniforms, Scissors};
use super::pipeline::GraphicsPipeline;
use super::fence::FenceValue;
use super::state_cache::StateCache;

use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;

use gl;
use gl::types::*;

/// A slice of a buffer that cannot be used outside the frame it has been allocated in.
/// This is statically prevented by the lifetime bound.
pub struct TransientBufferSlice<'a, T>
    where
        T: BufferData + ?Sized,
{
    // don't make this public: the user should not be able to extend the lifetime of slice
    slice: BufferSlice<T>,
    _phantom: PhantomData<&'a T>,
}

/// Trait that provides a method for getting a strong reference to the underlying resource
/// of a slice.
/// The method is unsafe because it allows extending resources beyond their static lifetime
/// bounds, and should only be used internally by `Frame`, which does its own synchronization.
/// Not meant to be implemented in user code.
/// Can't use deref, because an user could then accidentally extend the lifetime of
/// a TransientBufferSlice outside the frame.
pub unsafe trait ToRawBufferSlice {
    type Target: BufferData + ?Sized;
    unsafe fn to_raw_slice(&self) -> RawBufferSlice;
}

unsafe impl<'a,T> ToRawBufferSlice for TransientBufferSlice<'a,T> where T: BufferData + ?Sized
{
    type Target = T;
    unsafe fn to_raw_slice(&self) -> RawBufferSlice {
        self.slice.to_raw_slice()
    }
}

unsafe impl<T> ToRawBufferSlice for BufferSlice<T> where T: BufferData + ?Sized
{
    type Target = T;
    unsafe fn to_raw_slice(&self) -> RawBufferSlice {
        let clone = self.clone();
        clone.into_raw()
    }
}

/// A frame: instances are alive until the frame is complete
pub struct Frame<'q> {
    // Associated queue
    queue: &'q mut Queue,
    // Built-in upload buffer for convenience
    // Resources held onto by this frame
    pub(super) ref_buffers: RefCell<Vec<RawBuffer>>,
    pub(super) ref_textures: RefCell<Vec<TextureAny>>,
    pub(super) state_cache: RefCell<StateCache>,
}


impl<'q> Frame<'q> {
    /// Creates a new frame, mut-borrows the queue
    /// Since we can't build multiple command streams in parallel in OpenGL
    pub fn new<'a>(queue: &'a mut Queue) -> Frame<'a>
    {
        Frame {
            queue,
            //upload_buffer: UploadBuffer::new(queue.context(), DEFAULT_UPLOAD_BUFFER_SIZE),
            ref_textures: RefCell::new(Vec::new()),
            ref_buffers: RefCell::new(Vec::new()),
            state_cache: RefCell::new(StateCache::new())
        }
    }

    /// Allocates and uploads data to a *transient buffer* that live only until the GPU has finished using them.
    /// Can be used for uniform buffers, shader storage buffers, vertex buffers, etc.
    /// TODO specify target usage
    /// The lifetime of the returned resource is bound to the lifetime of self:
    /// this allows to statically limit the usage of the buffer to the current frame only.
    pub fn upload_into<'a, T: BufferData + ?Sized>(&'a self, upload_buffer: &'a UploadBuffer, data: &T) -> TransientBufferSlice<'a, T>
    {
        TransientBufferSlice {
            slice: unsafe {
                // TODO infer alignment from usage
                upload_buffer.upload(data, 256, self.queue.next_frame_fence_value(), self.queue.last_completed_frame())
            },
            _phantom: PhantomData
        }
    }


    /// Allocates and uploads data to the default upload buffer of the queue.
    /// Effectively calls `upload_into` with `self.queue().default_upload_buffer()`
    pub fn upload<'a, T: BufferData + ?Sized>(&'a self, data: &T) -> TransientBufferSlice<'a, T> {
        self.upload_into(self.queue.default_upload_buffer(), data)
    }

    /// Returns the current value of the fence of the queue.
    pub fn fence_value(&self) -> FenceValue {
        self.queue.fence.borrow().next_value()
    }

    /// Returns the queue of this frame.
    /// Note that you can't do any operations on the frame while the returned borrow
    /// still lives
    pub fn queue<'a>(&'a self) -> &'a Queue {
        self.queue
    }

    /// Consumes self, also releases the borrow on queue
    /// This function hands off all resources referenced during command stream construction
    /// to the queue, which will drop the references to the resources
    /// as soon as they are no longer in use by the GPU
    pub fn submit(self) {
        //debug!("Submit frame: sync index={:?}", self.queue.fence.borrow().next_value());
        // setup fence in command stream
        self.queue.submit(FrameResources {
            ref_buffers: self.ref_buffers.into_inner(),
            ref_textures: self.ref_textures.into_inner(),
        });
    }
}

