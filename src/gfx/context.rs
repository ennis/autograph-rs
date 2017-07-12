use gl::types::*;
use gl;
use std::ffi::CStr;
use std::mem;
use typed_arena::Arena;


extern "system" fn debug_callback(
    source: GLenum,
    ty: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    msg: *const GLchar,
    data: *mut GLvoid)
{

}

#[derive(Copy,Clone)]
pub struct ContextConfig
{
    pub max_frames_in_flight: i32,
    pub default_upload_buffer_size: i32
}

pub struct Context
{
    cfg: ContextConfig
}

impl Context
{
    pub fn new(cfg: &ContextConfig) -> Context {
        Context { cfg: *cfg }
    }
}
