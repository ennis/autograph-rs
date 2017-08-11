use gl::types::*;
use gl;
use std::sync::{Arc, Mutex};
use std::os::raw::c_void;
use std::str;
use std::slice;
use super::sampler::{Sampler, SamplerDesc};
use std::collections::HashMap;

extern "system" fn debug_callback(
    source: GLenum,
    ty: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    msg: *const GLchar,
    data: *mut GLvoid,
) {
    let str = unsafe {
        str::from_utf8(slice::from_raw_parts(msg as *const u8, length as usize)).unwrap()
    };
    debug!("GL: {}", str);
}

#[derive(Copy, Clone, Debug)]
pub struct ContextConfig {
    pub max_frames_in_flight: u32,
}

#[derive(Debug)]
pub struct Context {
    cfg: ContextConfig,
    sampler_cache: Mutex<HashMap<SamplerDesc, Arc<Sampler>>>,
}

impl Context {
    pub fn new(cfg: &ContextConfig) -> Arc<Context> {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(debug_callback as GLDEBUGPROC, 0 as *const c_void);
            gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                0 as *const u32,
                1,
            );
            gl::DebugMessageInsert(
                gl::DEBUG_SOURCE_APPLICATION,
                gl::DEBUG_TYPE_MARKER,
                1111,
                gl::DEBUG_SEVERITY_NOTIFICATION,
                -1,
                "Started logging OpenGL messages".as_ptr() as *const i8,
            );
        }

        Arc::new(Context {
            cfg: *cfg,
            sampler_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn get_sampler(&self, desc: &SamplerDesc) -> Arc<Sampler> {
        self.sampler_cache
            .lock()
            .unwrap()
            .entry(*desc)
            .or_insert_with(|| Arc::new(Sampler::new(desc)))
            .clone()
    }
}
