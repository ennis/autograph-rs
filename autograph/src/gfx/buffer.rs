use gl;
use gl::types::*;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use super::context::Context;
use std::sync::Arc;
use super::buffer_data::BufferData;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub struct RawBufferSlice {
    pub obj: GLuint,
    pub offset: usize,
    pub size: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BufferUsage {
    UPLOAD,
    DEFAULT,
    READBACK,
}

#[derive(Debug)]
pub struct Buffer<T: BufferData + ?Sized> {
    context: Arc<Context>,
    obj: GLuint,
    len: usize,
    usage: BufferUsage,
    _phantom: PhantomData<T>,
}


unsafe fn create_buffer<T: BufferData + ?Sized>(
    byte_size: usize,
    usage: BufferUsage,
    initial_data: Option<&T>,
) -> GLuint {
    let mut obj: GLuint = 0;
    unsafe {
        let flags = match usage {
            BufferUsage::READBACK => {
                gl::MAP_READ_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT
            }
            BufferUsage::UPLOAD => {
                gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT
            }
            BufferUsage::DEFAULT => 0,
        };
        gl::CreateBuffers(1, &mut obj);
        gl::NamedBufferStorage(
            obj,
            byte_size as isize,
            if let Some(data) = initial_data {
                data as *const T as *const GLvoid
            } else {
                0 as *const GLvoid
            },
            flags,
        );
    }
    obj
}

// Represents an OpenGL buffer without any type info
pub trait BufferAny {
    fn object(&self) -> GLuint;
}

pub struct BufferSlice<T: BufferData + ?Sized> {
    pub owner: Arc<BufferAny>,
    pub byte_offset: usize,
    pub len: usize, // number of elements
    _phantom: PhantomData<*const T>,
}

impl<T: BufferData + ?Sized> BufferSlice<T> {
    pub fn byte_size(&self) -> usize {
        self.len * mem::size_of::<T::Element>()
    }
}

// Untyped buffer slice
pub struct BufferSliceAny {
    pub owner: Arc<BufferAny>,
    pub byte_offset: usize,
    pub byte_size: usize,
}

impl BufferSliceAny {
    pub unsafe fn cast<T: BufferData + ?Sized>(self) -> BufferSlice<T> {
        BufferSlice {
            owner: self.owner,
            byte_offset: self.byte_offset,
            len: {
                let elem_size = mem::size_of::<T::Element>();
                assert!(self.byte_size % elem_size == 0);
                self.byte_size / elem_size
            },
            _phantom: PhantomData,
        }
    }
}

pub struct BufferMapping<T: BufferData + ?Sized> {
    pub owner: Arc<BufferAny>,
    pub ptr: *mut T,
    pub len: usize,
    _phantom: PhantomData<*const T>,
}

/*#[derive(Copy,Clone,Debug)]
pub struct BufferDesc {

}*/

impl<T: BufferData + ?Sized> Buffer<T> {
    pub fn new(ctx: &Arc<Context>, len: usize, usage: BufferUsage) -> Buffer<T> {
        Buffer {
            context: ctx.clone(),
            obj: unsafe { create_buffer::<T>(len * mem::size_of::<T::Element>(), usage, None) },
            len,
            usage,
            _phantom: PhantomData,
        }
    }

    pub fn with_data(ctx: &Arc<Context>, usage: BufferUsage, data: &T) -> Buffer<T> {
        let size = mem::size_of_val(data);
        Buffer {
            context: ctx.clone(),
            obj: unsafe { create_buffer(mem::size_of_val(data), usage, Some(data)) },
            len: data.len(),
            usage,
            _phantom: PhantomData,
        }
    }

    /*
    pub unsafe fn raw_slice(&self, offset: usize, size: usize) -> RawBufferSlice
    {
        assert!(offset + size <= self.size);
        RawBufferSlice { size, obj: self.obj, offset }
    }

    pub unsafe fn as_raw_slice(&self) -> RawBufferSlice
    {
        RawBufferSlice { size: self.size, obj: self.obj, offset: 0 }
    }
    */

    // TODO mut and non-mut functions
    pub unsafe fn map_persistent_unsynchronized(&self) -> *mut c_void {
        let mut flags = match self.usage {
            BufferUsage::READBACK => {
                gl::MAP_UNSYNCHRONIZED_BIT | gl::MAP_READ_BIT | gl::MAP_PERSISTENT_BIT |
                    gl::MAP_COHERENT_BIT
            }
            BufferUsage::UPLOAD => {
                gl::MAP_UNSYNCHRONIZED_BIT | gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT |
                    gl::MAP_COHERENT_BIT
            }
            BufferUsage::DEFAULT => {
                panic!("Cannot map a buffer allocated with BufferUsage::DEFAULT")
            }
        };

        gl::MapNamedBufferRange(self.obj, 0, self.byte_size() as isize, flags)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn byte_size(&self) -> usize {
        self.len * mem::size_of::<T::Element>()
    }
}

pub trait AsSlice<T: BufferData + ?Sized> {
    fn as_slice(&self) -> BufferSlice<T>;
    fn as_slice_any(&self) -> BufferSliceAny;
    unsafe fn get_slice_any(&self, byte_offset: usize, byte_size: usize) -> BufferSliceAny;
}

impl<T: BufferData + ?Sized> AsSlice<T> for Arc<Buffer<T>> {
    fn as_slice(&self) -> BufferSlice<T> {
        BufferSlice {
            owner: self.clone(),
            len: self.len,
            byte_offset: 0,
            _phantom: PhantomData,
        }
    }

    // Type-erased version of the above
    fn as_slice_any(&self) -> BufferSliceAny {
        BufferSliceAny {
            owner: self.clone(),
            byte_size: self.byte_size(),
            byte_offset: 0,
        }
    }

    unsafe fn get_slice_any(&self, byte_offset: usize, byte_size: usize) -> BufferSliceAny {
        // TODO check that the range is inside
        BufferSliceAny {
            owner: self.clone(),
            byte_size: byte_size,
            byte_offset: byte_offset,
        }
    }
}

// TODO Deref<Target=BufferSlice> for Arc<Buffer<T>>


impl<T: BufferData + ?Sized> BufferAny for Buffer<T> {
    fn object(&self) -> GLuint {
        self.obj
    }
}

impl<T: BufferData + ?Sized> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.obj);
        }
    }
}
