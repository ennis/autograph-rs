use gl::types::*;
use gl;
use std::ffi::CStr;
use std::mem;
use typed_arena::Arena;
use std::rc::Rc;
use std::cell::RefCell;
use std::os::raw::c_void;
use std::str;
use std::slice;
use super::sampler::{SamplerDesc,Sampler};
use std::collections::HashMap;


extern "system" fn debug_callback(
    source: GLenum,
    ty: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    msg: *const GLchar,
    data: *mut GLvoid)
{
    let str = unsafe {
        str::from_utf8(slice::from_raw_parts(msg as *const u8, length as usize)).unwrap()
    };
    debug!("GL: {}", str);
}

#[derive(Copy,Clone,Debug)]
pub struct ContextConfig
{
    pub max_frames_in_flight: u32,
    pub default_upload_buffer_size: usize
}

#[derive(Debug)]
pub struct Context
{
    cfg: ContextConfig,
    sampler_cache: RefCell<HashMap<SamplerDesc, Rc<Sampler>>>
}

impl Context
{
    pub fn new(cfg: &ContextConfig) -> Rc<Context> {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(debug_callback as GLDEBUGPROC, 0 as *const c_void);
            gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, 0 as *const u32, 1);
            gl::DebugMessageInsert(gl::DEBUG_SOURCE_APPLICATION, gl::DEBUG_TYPE_MARKER,
                                   1111, gl::DEBUG_SEVERITY_NOTIFICATION, -1,
                                   "Started logging OpenGL messages".as_ptr() as *const i8);
        }

        Rc::new(Context { cfg: *cfg, sampler_cache: RefCell::new(HashMap::new())})
    }

    pub fn get_sampler(&self, desc: &SamplerDesc) -> Rc<Sampler>
    {
        self.sampler_cache.borrow_mut().entry(*desc).or_insert_with(|| Rc::new(Sampler::new(desc))).clone()
    }
}


