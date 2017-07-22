use gl;
use gl::types::*;
use smallvec::SmallVec;

struct VertexAttribute {
    pub slot: i32,
    pub type_: GLenum,
    pub size: isize,
    pub relativeOffset: isize,
    pub normalized: bool
}

struct VertexArray
{
    attribs: SmallVec<[VertexAttribute; 8]>,
    obj: GLuint
}

impl VertexArray
{
    pub fn with_attribs(attribs: &[VertexAttribute]) {

    }
}

impl Drop for VertexArray
{
    fn drop(&mut self) {
        if self.obj != 0 {
            unsafe {
                gl::DeleteVertexArrays(1, &self.obj);
            }
        }
    }
}