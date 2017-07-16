use std::os::raw::{c_char, c_uchar, c_uint};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AiTexel {
    pub b: c_uchar,
    pub g: c_uchar,
    pub r: c_uchar,
    pub a: c_uchar,
}

#[repr(C)]
pub struct AiTexture {
    pub width: c_uint,
    pub height: c_uint,
    pub format_hint: [c_char; 4],
    pub data: *mut AiTexel,
}
