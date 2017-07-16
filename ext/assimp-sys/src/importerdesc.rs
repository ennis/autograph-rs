use std::os::raw::{c_char, c_uint};

bitflags! {
    #[repr(C)]
    flags AiImporterFlags: c_uint {
        const AIIMPORTERFLAG_SUPPORT_TEXT_FLAVOUR = 0x1,
        const AIIMPORTERFLAG_SUPPORT_BINARY_FLAVOUR = 0x2,
        const AIIMPORTERFLAG_SUPPORT_COMPRESSED_FLAVOUR = 0x4,
        const AIIMPORTERFLAG_LIMITED_SUPPORT = 0x8,
        const AIIMPORTERFLAG_EXPERIMENTAL = 0x10
    }
}

#[repr(C)]
pub struct AiImporterDesc {
    pub name: *const c_char,
    pub author: *const c_char,
    pub maintainer: *const c_char,
    pub comments: *const c_char,
    pub flags: AiImporterFlags,
    pub min_major: c_uint,
    pub min_minor: c_uint,
    pub max_major: c_uint,
    pub max_minor: c_uint,
    pub file_extensions: *const c_char,
}

extern {
    pub fn aiGetImporterDesc(
        extension: *const c_char) -> *const AiImporterDesc;
}
