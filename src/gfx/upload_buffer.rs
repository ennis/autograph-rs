use gl;
use gl::types::*;
use std::slice;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use super::context::Context;
use super::buffer::{Buffer, RawBufferSlice, BufferUsage};
use std::collections::vec_deque::VecDeque;
use super::frame::Frame;
use std::rc::Rc;

struct FencedRegion
{
    expiration: i64,
    begin_ptr: usize,
    end_ptr: usize
}

pub struct UploadBufferState
{
    write: usize,
    begin: usize,
    used: usize,
    fenced_regions: VecDeque<FencedRegion>
}

pub struct UploadBuffer
{
    buffer: Rc<Buffer>, // Owned
    state: RefCell<UploadBufferState>,
    mapped_region: *mut c_void,
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

//
pub struct TransientBuffer<'frame>
{
    buffer: Rc<Buffer>,
    frame_id: i64,
    slice: RawBufferSlice,
    _phantom: PhantomData<&'frame ()>
}

// TODO: impl trait 'PipelineBindable' for TransientBuffer


impl UploadBuffer
{
    pub fn new(ctx: Rc<Context>, buffer_size: usize) -> UploadBuffer
    {
        //UploadBuffer { _phantom: PhantomData }
        let buffer = Rc::new(Buffer::new(ctx.clone(), buffer_size, BufferUsage::UPLOAD));
        let mapped_region = unsafe {
            buffer.map_persistent_unsynchronized()
        };

        UploadBuffer {
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
    //
    // draw pipelines should NOT consume a transient buffer slice
    //
    // Binding a buffer to the pipeline: take a Rc<Buffer> + Buffer slice
    //
    // TL;DR 3 - Conclusion: Resources bound to the pipeline must be Rc, since they escape all known static lifetimes

    // TODO the output slice should only be valid during the current frame:
    // maybe pass a reference to the frame object and bound the lifetime of the slice
    // to the lifetime of the frame?
    // The output slice is then 'consumed'
    pub fn upload<'frame, T: Copy>(&self, frame: &'frame Frame, data: &T, align: usize) -> TransientBuffer<'frame>
    {
        TransientBuffer {
            _phantom: PhantomData,
            buffer: self.buffer.clone(),
            frame_id: frame.id,
            slice: unsafe { self.allocate(mem::size_of::<T>(), align, frame.id).expect("Upload buffer is full") }
        }
    }

    pub fn upload_slice<'frame, T: Copy>(&self, frame: &'frame Frame, data: &[T]) -> TransientBuffer<'frame>
    {
        TransientBuffer {
            _phantom: PhantomData,
            buffer: self.buffer.clone(),
            frame_id: frame.id,
            slice: unsafe { self.allocate(data.len() * mem::size_of::<T>(), 16, frame.id).expect("Upload buffer is full") }
        }
    }

    unsafe fn allocate<'a>(&'a self, size: usize, align: usize, expiration: i64) -> Option<RawBufferSlice>
    {
        if let Some(offset) = self.try_allocate_contiguous(expiration, size, align) {
            Some(self.buffer.get_raw_slice(offset, size))
        } else {
            None
        }
    }

    fn try_allocate_contiguous(&self, expiration: i64, size: usize, align: usize) -> Option<usize> {
        //assert!(size < self.buffer.size);
        let mut state = self.state.borrow_mut();

        if (state.begin < state.write) || (state.begin == state.write && state.used == 0) {
            let slack = self.buffer.size() - state.write;
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
        state.fenced_regions.push_back(FencedRegion{ begin_ptr: alloc_begin, end_ptr: alloc_begin + size, expiration });
        Some(alloc_begin)
    }

    // TODO decide who should call it?
    fn reclaim(&self, date: i64) {
        let mut state = self.state.borrow_mut();
         while !state.fenced_regions.is_empty() && state.fenced_regions.front().unwrap().expiration <= date {
             let region = state.fenced_regions.pop_front().unwrap();
             state.begin = region.end_ptr;
             state.used -= region.end_ptr - region.begin_ptr;
         }
    }


    // uploadBuffer deals slices of its internal buffer, however, they live as long as the frame they
    // are allocated into
    // buf we don't want an exclusive borrow of the UploadBuffer, which would be useless: use a refcell internally
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