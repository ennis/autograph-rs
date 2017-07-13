use gl;
use gl::types::*;
use std::slice;
use libc::c_void;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use super::context::Context;
use super::buffer::{Buffer, BufferSlice, BufferUsage};
use std::collections::vec_deque::VecDeque;
use super::frame::Frame;

struct FencedRegion
{
    expiration: i64,
    begin_ptr: isize,
    end_ptr: isize
}

pub struct UploadBufferState
{
    write: isize,
    begin: isize,
    used: isize,
    fenced_regions: VecDeque<FencedRegion>
}

pub struct UploadBuffer<'ctx>
{
    buffer: Buffer<'ctx>,
    state: RefCell<UploadBufferState>,
    mapped_region: *mut c_void,
    _phantom: PhantomData<&'ctx ()>
}

fn align_offset(align: isize, size: isize, ptr: isize, space: isize) -> Option<isize>
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

impl<'ctx> UploadBuffer<'ctx>
{
    pub fn new<'a>(ctx: &'a Context, buffer_size: i64) -> UploadBuffer<'a>
    {
        //UploadBuffer { _phantom: PhantomData }
        unimplemented!()
    }

    // TODO the output slice should only be valid during the current frame:
    // maybe pass a reference to the frame object and bound the lifetime of the slice
    // to the lifetime of the frame?
    pub fn upload<'frame, 'a: 'frame, T: Copy>(&'a self, frame: &'frame Frame, data: &T, align: isize) -> BufferSlice<'frame>
    {
        /*if let Some(slice) = self.allocate(mem::size_of::<T>(), align) {

        } else {

        }*/
        unimplemented!()
    }

    pub fn upload_slice<'a, T: Copy>(&'a self, data: &[T]) -> BufferSlice<'a>
    {
        unimplemented!()
    }

    fn allocate<'a>(&'a self, size: isize, align: isize, expiration: i64) -> Option<BufferSlice<'a>>
    {
        if let Some(offset) = self.try_allocate_contiguous(expiration, size, align) {
            Some(self.buffer.get_slice(offset, size))
        } else {
            None
        }
    }

    fn try_allocate_contiguous(&self, expiration: i64, size: isize, align: isize) -> Option<isize> {
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
    let ctx: Context = unimplemented!();
    let frame: Frame = unimplemented!();    // 'frame
    let uploadbuf = UploadBuffer::new(&ctx, 3 * 1024 * 1024);
    let u0 = uploadbuf.upload(&frame, &0, 16);
    let u1 = uploadbuf.upload(&frame, &1, 16);
    // upload buf drops here
    // frame drops here
    // ctx drops here
}