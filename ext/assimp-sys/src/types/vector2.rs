use std::os::raw::c_float;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiVector2D {
    pub x: c_float,
    pub y: c_float,
}
