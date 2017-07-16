use std::os::raw::c_float;

use types::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiCamera {
    pub name: AiString,
    pub position: AiVector3D,
    pub up: AiVector3D,
    pub look_at: AiVector3D,
    pub horizontal_fov: c_float,
    pub clip_plane_near: c_float,
    pub clip_plane_far: c_float,
    pub aspect: c_float,
}
