use std::os::raw::{c_float, c_uint};

// Reexport submodules
pub use self::color3::*;
pub use self::color4::*;
pub use self::matrix3::*;
pub use self::matrix4::*;
pub use self::quaternion::*;
pub use self::string::*;
pub use self::vector2::*;
pub use self::vector3::*;

mod color3;
mod color4;
mod matrix3;
mod matrix4;
mod quaternion;
mod string;
mod vector2;
mod vector3;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiPlane {
    pub a: c_float,
    pub b: c_float,
    pub c: c_float,
    pub d: c_float,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiRay {
    pub pos: AiVector3D,
    pub dir: AiVector3D,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiReturn {
    Success = 0,
    Failure = 1,
    OutOfMemory = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiOrigin {
    Set = 0,
    Cur = 1,
    End = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiDefaultLogStream {
    File = 1,
    StdOut = 2,
    StdErr = 4,
    Debugger = 8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AiMemoryInfo {
    pub textures: c_uint,
    pub materials: c_uint,
    pub meshes: c_uint,
    pub nodes: c_uint,
    pub animations: c_uint,
    pub cameras: c_uint,
    pub lights: c_uint,
    pub total: c_uint,
}
