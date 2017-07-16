use std::os::raw::{c_char, c_uint};

extern {
    pub fn aiGetLegalString() -> *const c_char;
    pub fn aiGetVersionMinor() -> c_uint;
    pub fn aiGetVersionMajor() -> c_uint;
    pub fn aiGetVersionRevision() -> c_uint;
    pub fn aiGetCompileFlags() -> AiCompileFlags;
}

bitflags! {
    #[repr(C)]
    flags AiCompileFlags: c_uint {
        const ASSIMP_CFLAGS_SHARED = 0x1,
        const ASSIMP_CFLAGS_STLPORT = 0x2,
        const ASSIMP_CFLAGS_DEBUG = 0x4,
        const ASSIMP_CFLAGS_NOBOOST = 0x8,
        const ASSIMP_CFLAGS_SINGLETHREADED = 0x10
    }
}
