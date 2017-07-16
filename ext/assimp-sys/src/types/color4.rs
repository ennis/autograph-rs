use std::os::raw::c_float;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiColor4D {
    pub r: c_float,
    pub g: c_float,
    pub b: c_float,
    pub a: c_float,
}
