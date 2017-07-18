use gl;
use gl::types::*;
use std::slice;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use std::os::raw::c_void;
use super::context::Context;
use std::rc::Rc;

#[derive(Copy,Clone)]
pub struct BufferSlice<'buf>
{
    obj: GLuint,
    offset: isize,
    size: isize,
    _phantom: PhantomData<&'buf ()>
}

#[derive(Copy,Clone,PartialEq)]
pub enum BufferUsage
{
    UPLOAD,
    DEFAULT,
    READBACK
}

pub struct Buffer
{
    obj: GLuint,
    size: isize,
    usage: BufferUsage
}

/*void *Buffer::map(size_t offset, size_t size) {
gl::GLbitfield flags = gl::MAP_UNSYNCHRONIZED_BIT;
if (usage_ == Usage::Readback) {
flags |= gl::MAP_READ_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
} else if (usage_ == Usage::Upload) {
flags |= gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
} else {
// cannot map a DEFAULT buffer
throw std::logic_error(
"Trying to map a buffer allocated with gl_buffer_usage::Default");
}
return gl::MapNamedBufferRange(object(), offset, size, flags);
}

Buffer::Buffer(std::size_t byteSize, Buffer::Usage usage,
const void *initial_data)
: usage_{usage}, byte_size_{byteSize} {
gl::GLbitfield flags = 0;
if (usage == Usage::Readback) {
flags |= gl::MAP_READ_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
} else if (usage == Usage::Upload) {
flags |= gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
} else {
flags = 0;
}

gl::GLuint buf_obj;
gl::CreateBuffers(1, &buf_obj);
gl::NamedBufferStorage(buf_obj, byteSize, initial_data, flags);
obj_ = buf_obj;
}*/


impl Buffer
{
    pub fn new(ctx: Rc<Context>, size: isize, usage: BufferUsage) -> Buffer
    {
        let mut obj: GLuint = 0;
        unsafe {
            let flags = if usage == BufferUsage::READBACK {
                gl::MAP_READ_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT
            } else if usage ==BufferUsage::UPLOAD {
                gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT
            } else {
                0
            };
            gl::CreateBuffers(1, &mut obj);
            gl::NamedBufferStorage(obj, size, 0 as *const c_void, flags);
        }
        Buffer { obj, size, usage }
    }

    pub fn size(&self) -> isize { self.size }

    pub fn as_slice<'buf>(&'buf self) -> BufferSlice<'buf> { BufferSlice { size: self.size, obj: self.obj, offset: 0, _phantom: PhantomData } }

    pub fn get_slice<'buf>(&'buf self, offset: isize, size: isize) -> BufferSlice<'buf> {
        assert!(offset + size <= self.size);
        BufferSlice { size, obj: self.obj, offset, _phantom: PhantomData }
    }
}

