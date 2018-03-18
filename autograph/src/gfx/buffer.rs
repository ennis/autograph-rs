use gl;
use gl::types::*;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use super::context::Context;
use std::sync::Arc;
use super::buffer_data::BufferData;
use std::clone::Clone;
use std::ops::Deref;
use gfx::shader_interface::VertexType;

#[derive(Copy, Clone, Debug)]
pub(super) struct RawBufferSliceGL {
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
pub struct RawBufferObject {
    gctx: Context,
    obj: GLuint,
    byte_size: usize,
    usage: BufferUsage
}

unsafe fn create_buffer<T: BufferData + ?Sized>(
    byte_size: usize,
    usage: BufferUsage,
    initial_data: Option<&T>,
) -> GLuint {
    let mut obj: GLuint = 0;
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

    obj
}

#[derive(Clone,Debug)]
pub struct RawBuffer(Arc<RawBufferObject>);

#[derive(Clone, Debug)]
pub struct RawBufferSlice {
    pub owner: RawBuffer,
    pub offset: usize,
    pub byte_size: usize,
}

impl RawBufferSlice {
    pub unsafe fn into_typed<T: BufferData + ?Sized>(self) -> BufferSlice<T> {
        let elem_size = mem::size_of::<T::Element>();
        assert!(self.byte_size % elem_size == 0);
        BufferSlice {
            raw: self,
            _phantom: PhantomData
        }
    }
}


#[derive(Debug)]
pub struct BufferSlice<T: BufferData + ?Sized> {
    pub raw: RawBufferSlice,
    _phantom: PhantomData<*const T>,
}

// Explicit impl of Clone, workaround issue 26925 ?
// https://github.com/rust-lang/rust/issues/26925
impl<T: BufferData + ?Sized> Clone for BufferSlice<T> {
    fn clone(&self) -> Self {
        BufferSlice {
            raw: self.raw.clone(),
            _phantom: PhantomData
        }
    }
}

impl<T: BufferData + ?Sized> BufferSlice<T> {
    pub fn len(&self) -> usize {
        self.raw.byte_size / mem::size_of::<T::Element>()
    }

    pub fn byte_size(&self) -> usize {
        self.raw.byte_size
    }

    pub fn into_raw(self) -> RawBufferSlice {
        self.raw
    }
}

pub struct BufferMapping<T: BufferData + ?Sized> {
    pub owner: RawBuffer,
    pub ptr: *mut T,
    pub len: usize,
    _phantom: PhantomData<*const T>,
}


impl RawBufferObject {
    pub fn new(gctx: &Context, byte_size: usize, usage: BufferUsage) -> RawBufferObject {
        RawBufferObject {
            gctx: gctx.clone(),
            obj: unsafe { create_buffer::<u8>(byte_size, usage, None) },
            byte_size,
            usage
        }
    }

    pub fn with_data<T: BufferData + ?Sized>(gctx: &Context, usage: BufferUsage, data: &T) -> RawBufferObject {
        let byte_size = mem::size_of_val(data);
        RawBufferObject {
            gctx: gctx.clone(),
            obj: unsafe { create_buffer(mem::size_of_val(data), usage, Some(data)) },
            byte_size,
            usage
        }
    }

    // TODO mut and non-mut functions
    pub unsafe fn map_persistent_unsynchronized(&self) -> *mut c_void {
        let flags = match self.usage {
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

    pub fn gl_object(&self) -> GLuint {
        self.obj
    }

    pub fn byte_size(&self) -> usize {
        self.byte_size
    }
}

impl RawBuffer
{
    pub fn new(gctx: &Context, byte_size: usize, usage: BufferUsage) -> RawBuffer {
        RawBuffer(Arc::new(RawBufferObject::new(gctx, byte_size, usage)))
    }

    pub fn with_data<T: BufferData + ?Sized>(gctx: &Context, usage: BufferUsage, data: &T) -> RawBuffer {
        RawBuffer(Arc::new(RawBufferObject::with_data(gctx, usage, data)))
    }

    // This is unsafe because nothing prevents the user from creating two overlapping
    // slices, and immutability of the buffer contents is not enforced by this API
    pub unsafe fn get_slice(&self, offset: usize, byte_size: usize) -> RawBufferSlice {
        RawBufferSlice {
            owner: self.clone(),
            byte_size,
            offset
        }
    }

    pub fn gl_object(&self) -> GLuint {
        self.0.gl_object()
    }

    pub fn byte_size(&self) -> usize {
        self.0.byte_size
    }
}

impl Deref for RawBuffer
{
    type Target = RawBufferObject;
    fn deref(&self) -> &RawBufferObject {
        &self.0
    }
}

#[derive(Clone,Debug)]
pub struct Buffer<T: BufferData + ?Sized>(RawBuffer, PhantomData<*const T>);

impl<T: BufferData + ?Sized> Deref for Buffer<T>
{
    type Target = RawBuffer;
    fn deref(&self) -> &RawBuffer {
        &self.0
    }
}

impl<T: BufferData+?Sized> Buffer<T>
{
    pub fn new(gctx: &Context, byte_size: usize, usage: BufferUsage) -> Buffer<T> {
        Buffer(RawBuffer(Arc::new(RawBufferObject::new(gctx, byte_size, usage))), PhantomData)
    }

    pub fn with_data(gctx: &Context, usage: BufferUsage, data: &T) -> Buffer<T> {
        Buffer(RawBuffer(Arc::new(RawBufferObject::with_data(gctx, usage, data))), PhantomData)
    }
}

#[derive(Clone,Debug)]
pub struct VertexBuffer<T: VertexType + ?Sized>(Buffer<T>);

impl<T: VertexType + ?Sized> VertexBuffer<T> {
    pub fn new(gctx: &Context, byte_size: usize, usage: BufferUsage) -> VertexBuffer<T> {
         VertexBuffer(Buffer::new(gctx, byte_size, usage))
    }

    pub fn with_data(gctx: &Context, usage: BufferUsage, data: &T) -> VertexBuffer<T> {
        VertexBuffer(Buffer::with_data(gctx, usage, data))
    }
}

impl<T: VertexType + ?Sized> Deref for VertexBuffer<T>
{
    type Target = Buffer<T>;
    fn deref(&self) -> &Buffer<T> {
        &self.0
    }
}


/*pub trait AsSlice<T: BufferData + ?Sized> {
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
}*/

impl Drop for RawBufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.obj);
        }
    }
}
