use std::os::raw::{c_double, c_uint};

use types::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiVectorKey {
    pub time: c_double,
    pub value: AiVector3D,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiQuatKey {
    pub time: c_double,
    pub value: AiQuaternion,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiMeshKey {
    pub time: c_double,
    pub value: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiAnimBehaviour {
    Default = 0,
    Constant = 1,
    Linear = 2,
    Repeat = 3,
}

#[repr(C)]
pub struct AiNodeAnim {
    pub node_name: AiString,
    pub num_position_keys: c_uint,
    pub position_keys: *mut AiVectorKey,
    pub num_rotation_keys: c_uint,
    pub rotation_keys: *mut AiQuatKey,
    pub num_scaling_keys: c_uint,
    pub scaling_keys: *mut AiVectorKey,
    pub pre_state: AiAnimBehaviour,
    pub post_state: AiAnimBehaviour,
}

#[repr(C)]
pub struct AiMeshAnim {
    pub name: AiString,
    pub num_keys: c_uint,
    pub keys: *mut AiMeshKey,
}

#[repr(C)]
pub struct AiAnimation {
    pub name: AiString,
    pub duration: c_double,
    pub ticks_per_second: c_double,
    pub num_channels: c_uint,
    pub channels: *mut *mut AiNodeAnim,
    pub num_mesh_channels: c_uint,
    pub mesh_channels: *mut *mut AiMeshAnim,
}
