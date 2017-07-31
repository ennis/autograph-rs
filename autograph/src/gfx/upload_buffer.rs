use gl;
use gl::types::*;
use std::slice;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use super::context::Context;
use super::buffer_data::BufferData;
use super::buffer::{Buffer, BufferSlice, BufferSliceAny, BufferUsage, AsSlice};
use std::collections::vec_deque::VecDeque;
use super::queue::{FrameQueue, Frame};
use std::rc::Rc;
use super::fence::{Fence, FenceValue};
use std::ptr::copy_nonoverlapping;
use std::ops::Deref;

struct FencedRegion
{
    fence_value: FenceValue,
    begin_ptr: usize,
    end_ptr: usize
}

pub struct UploadBufferState
{
    write: usize,
    begin: usize,
    used: usize,
    fenced_regions: VecDeque<FencedRegion>,
    // TODO frame fences
    //frame_fences:
}

pub struct UploadBuffer<'queue>
{
    queue: &'queue FrameQueue,
    buffer: Rc<Buffer<[u8]>>, // Owned
    state: RefCell<UploadBufferState>,
    mapped_region: *mut u8,
}

fn align_offset(align: usize, size: usize, ptr: usize, space: usize) -> Option<usize>
{
    let mut off = ptr & (align - 1);
    if off > 0 {
        off = align - off;
    };
    if space < off || space - off < size {
        None
    } else {
        Some(ptr+off)
    }
}

// Should this be Deref<Target=Buffer> ?
// TODO: add a way to get alignment of the slice
// Deref target=BufferSlice<T>
pub struct TransientBuffer<'frame, T> where T: BufferData + ?Sized
{
    slice: BufferSlice<T>,
    _phantom: PhantomData<&'frame T>
}

impl<'frame,T: BufferData + ?Sized> Deref for TransientBuffer<'frame,T>
{
    type Target = BufferSlice<T>;
    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<'queue> UploadBuffer<'queue>
{
    pub fn new<'a>(queue: &'a FrameQueue, buffer_size: usize) -> UploadBuffer<'a>
    {
        //UploadBuffer { _phantom: PhantomData }
        let buffer = Rc::new(Buffer::new(queue.context(), buffer_size, BufferUsage::UPLOAD));
        let mapped_region = unsafe {
            buffer.map_persistent_unsynchronized() as *mut u8
        };

        UploadBuffer {
            queue,
            buffer,
            state: RefCell::new(UploadBufferState {
                begin: 0,
                used: 0,
                write: 0,
                fenced_regions: VecDeque::new()
            }),
            mapped_region
        }
    }

    //
    // The output slice should be valid until the GPU has finished rendering the current frame
    // However, we do not expose a dynamic lifetime to the user: we simply say that
    // the buffer slice can be used until the frame is finalized.
    //
    // TL;DR
    // There are two lifetimes:
    // - logical: slice accessible until the frame is finalized
    // - actual: until the GPU has finished rendering the frame and the resources are reclaimed
    //
    // Issue:
    // Currently, nothing prevents a user from passing a buffer slice that **actually** lives
    // only during the current frame, and not long enough for the GPU operation to complete
    // => extend the lifetime of the buffer by passing an Rc<Buffer> into the buffer slice?
    //
    // Add an Rc<stuff> into the frame each time a resource is referenced in the frame
    //
    // TL;DR: the problem is that any resource reference passed into the GPU pipeline
    // escapes all known static lifetimes
    // Possible solution: bind the lifetime of all 'transient objects' to a GPUFuture object
    // Dropping the GPUFuture object means waiting for the frame to be finished
    // Thus, logical lifetime of the fence = actual lifetime of a frame
    //
    // TL;DR 2: 'Outliving the frame object' is not enough for resources
    // buffer slice of an Rc<Buffer> lives as long as the Rc => should live as long as the buffer?
    // draw pipelines should NOT consume a transient buffer slice
    // Binding a buffer to the pipeline: take a Rc<Buffer> + Buffer slice
    //
    // TL;DR 3 - Conclusion: Resources bound to the pipeline must be Rc, since they escape all known static lifetimes

    pub fn upload<'frame, T: BufferData + ?Sized>(&self, frame: &'frame Frame, data: &T, align: usize) -> TransientBuffer<'frame, T>
    {
        let byte_size = mem::size_of_val(data);
        let ptr = data as *const T as *const u8;
        assert!(frame.queue() as *const _ == self.queue as *const _, "UploadBuffer allocating on invalid queue");
        TransientBuffer {
            _phantom: PhantomData,
            slice: unsafe {
                let slice = self.allocate(byte_size, align, self.queue.next_frame_fence_value()).expect("Upload buffer is full");
                copy_nonoverlapping(ptr, self.mapped_region.offset(slice.byte_offset as isize), byte_size);
                slice.cast::<T>()
            }
        }
    }

    unsafe fn allocate(&self, size: usize, align: usize, fence_value: FenceValue) -> Option<BufferSliceAny>
    {
        debug!("alloc size={}, align={}, fence_value={:?}", size, align, fence_value);
        if let Some(offset) = self.try_allocate_contiguous(size, align, fence_value) {
            Some(self.buffer.get_slice_any(offset, size))
        } else {
            // reclaim and try again (not enough contiguous free space)
            self.reclaim();
            if let Some(offset) = self.try_allocate_contiguous(size, align, fence_value) {
                Some(self.buffer.get_slice_any(offset, size))
            }
            else {
                None
            }
        }
    }

    fn try_allocate_contiguous(&self, size: usize, align: usize, fence_value: FenceValue) -> Option<usize> {
        //assert!(size < self.buffer.size);
        let mut state = self.state.borrow_mut();

        if (state.begin < state.write) || (state.begin == state.write && state.used == 0) {
            let slack = self.buffer.byte_size() - state.write;
            // try to put the buffer in the slack space at the end
            if let Some(newptr) = align_offset(align, size, state.write, slack) {
                state.write = newptr;
            } else {
                // else, try to put it at the beginning (which is always correctly
                // aligned)
                if size > state.begin {
                    return None;
                }
                state.write = 0;
            }
        } else {
            // begin_ptr > write_ptr
            // reclaim space in the middle
            if let Some(newptr) = align_offset(align, size, state.write, state.begin - state.write) {
                state.write = newptr;
            }
            else {
                return None;
            }
        }

        let alloc_begin = state.write;
        state.used += size;
        state.write += size;
        state.fenced_regions.push_back(FencedRegion{ begin_ptr: alloc_begin, end_ptr: alloc_begin + size, fence_value });
        Some(alloc_begin)
    }

    fn reclaim(&self) {
        let last_completed_frame = self.queue.last_completed_frame();
        //debug!("reclaiming: last_completed_frame={:?}", last_completed_frame);
        let mut state = self.state.borrow_mut();
        while !state.fenced_regions.is_empty() && state.fenced_regions.front().unwrap().fence_value <= last_completed_frame {
            let region = state.fenced_regions.pop_front().unwrap();
            //debug!("reclaiming region {}-{} because a later frame was completed (region={:?} < last_completed_frame={:?})", region.begin_ptr, region.end_ptr, region.fence_value, last_completed_frame);
            state.begin = region.end_ptr;
            state.used -= region.end_ptr - region.begin_ptr;
        }
    }
}


#[test]
#[should_panic]
fn test_upload_buffer_lifetimes()
{
    let ctx: Rc<Context> = unimplemented!();
    let frame: Frame = unimplemented!();    // 'frame
    let uploadbuf = UploadBuffer::new(ctx.clone(), 3 * 1024 * 1024);
    let u0 = uploadbuf.upload(&frame, &0, 16);
    let u1 = uploadbuf.upload(&frame, &1, 16);
    // upload buf drops here
    // frame drops here
    // ctx drops here
}