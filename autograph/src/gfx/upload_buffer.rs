use super::buffer::{BufferAny, BufferSlice, BufferSliceAny, BufferUsage};
use super::buffer_data::BufferData;
use super::context::Context;
use super::fence::FenceValue;
use std::cell::RefCell;
use std::collections::vec_deque::VecDeque;
use std::mem;
use std::ptr::copy_nonoverlapping;

struct FencedRegion {
    fence_value: FenceValue,
    begin_ptr: usize,
    end_ptr: usize,
}

pub struct UploadBufferState {
    write: usize,
    begin: usize,
    used: usize,
    fenced_regions: VecDeque<FencedRegion>,
    // TODO frame fences
    //frame_fences:
}

pub struct UploadBuffer {
    buffer: BufferAny, // Owned
    state: RefCell<UploadBufferState>,
    mapped_region: *mut u8,
}

fn align_offset(align: usize, size: usize, ptr: usize, space: usize) -> Option<usize> {
    let mut off = ptr & (align - 1);
    if off > 0 {
        off = align - off;
    };
    if space < off || space - off < size {
        None
    } else {
        Some(ptr + off)
    }
}

impl UploadBuffer {
    pub fn new(gctx: &Context, buffer_size: usize) -> UploadBuffer {
        //UploadBuffer { _phantom: PhantomData }
        let buffer = BufferAny::new(gctx, buffer_size, BufferUsage::UPLOAD);
        let mapped_region = unsafe { buffer.map_persistent_unsynchronized() as *mut u8 };

        UploadBuffer {
            buffer,
            state: RefCell::new(UploadBufferState {
                begin: 0,
                used: 0,
                write: 0,
                fenced_regions: VecDeque::new(),
            }),
            mapped_region,
        }
    }

    /// Unsafe because the transient buffer can be reclaimed at any time with `reclaim`
    /// according to the `reclaim_until` parameter.
    /// For a safe implementation, allocate through Frame
    /// TODO do not panic if the upload buffer is full
    pub unsafe fn upload<T: BufferData + ?Sized>(
        &self,
        data: &T,
        align: usize,
        fence_value: FenceValue,
        reclaim_until: FenceValue,
    ) -> BufferSlice<T> {
        let byte_size = mem::size_of_val(data);
        let ptr = data as *const T as *const u8;
        let slice = self
            .allocate(byte_size, align, fence_value, reclaim_until)
            .expect("upload buffer is full"); // TODO expand? wait?
        copy_nonoverlapping(
            ptr,
            self.mapped_region.offset(slice.offset as isize),
            byte_size,
        );
        slice.into_typed::<T>()
    }

    /// Unsafe for the same reasons as `upload`
    unsafe fn allocate(
        &self,
        size: usize,
        align: usize,
        fence_value: FenceValue,
        reclaim_until: FenceValue,
    ) -> Option<BufferSliceAny> {
        //debug!("alloc size={}, align={}, fence_value={:?}", size, align, fence_value);
        if let Some(offset) = self.try_allocate_contiguous(size, align, fence_value) {
            Some(self.buffer.get_slice(offset, size))
        } else {
            // reclaim and try again (not enough contiguous free space)
            self.reclaim(reclaim_until);
            if let Some(offset) = self.try_allocate_contiguous(size, align, fence_value) {
                Some(self.buffer.get_slice(offset, size))
            } else {
                None
            }
        }
    }

    fn try_allocate_contiguous(
        &self,
        size: usize,
        align: usize,
        fence_value: FenceValue,
    ) -> Option<usize> {
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
            if let Some(newptr) = align_offset(align, size, state.write, state.begin - state.write)
            {
                state.write = newptr;
            } else {
                return None;
            }
        }

        let alloc_begin = state.write;
        state.used += size;
        state.write += size;
        state.fenced_regions.push_back(FencedRegion {
            begin_ptr: alloc_begin,
            end_ptr: alloc_begin + size,
            fence_value,
        });
        Some(alloc_begin)
    }

    fn reclaim(&self, reclaim_until: FenceValue) {
        //debug!("reclaiming: last_completed_fence_step={:?}", last_completed_fence_step);
        let mut state = self.state.borrow_mut();
        while !state.fenced_regions.is_empty()
            && state.fenced_regions.front().unwrap().fence_value <= reclaim_until
        {
            let region = state.fenced_regions.pop_front().unwrap();
            //debug!("reclaiming region {}-{} because all commands using these regions have completed (region={:?} < last_completed_fence_step={:?})", region.begin_ptr, region.end_ptr, region.fence_value, last_completed_fence_step);
            state.begin = region.end_ptr;
            state.used -= region.end_ptr - region.begin_ptr;
        }
    }
}

#[test]
#[should_panic]
fn test_upload_buffer_lifetimes() {
    /*let ctx: Arc<Context> = unimplemented!();
    let frame: Frame = unimplemented!();    // 'frame
    let uploadbuf = UploadBuffer::new(ctx.clone(), 3 * 1024 * 1024);
    let u0 = uploadbuf.upload(&frame, &0, 16);
    let u1 = uploadbuf.upload(&frame, &1, 16);*/
    // upload buf drops here
    // frame drops here
    // ctx drops here
}
