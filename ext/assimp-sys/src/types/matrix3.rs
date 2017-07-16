use std::os::raw::c_float;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiMatrix3x3 {
    pub a1: c_float,
    pub a2: c_float,
    pub a3: c_float,
    pub b1: c_float,
    pub b2: c_float,
    pub b3: c_float,
    pub c1: c_float,
    pub c2: c_float,
    pub c3: c_float,
}
