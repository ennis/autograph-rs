use std::os::raw::{c_uint, c_void};

use types::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiMetadataType {
    Bool = 0,
    Int = 1,
    Uint64 = 2,
    Float = 3,
    AiString = 4,
    AiVector3D = 5,
}

#[repr(C)]
pub struct AiMetadataEntry {
    pub data_type: AiMetadataType,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct AiMetadata {
    pub num_properties: c_uint,
    pub keys: *mut AiString,
    pub values: *mut AiMetadataEntry,
}
