use std::os::raw::{c_char, c_float, c_int, c_uint};

use types::*;

pub static AI_DEFAULT_MATERIAL_NAME: &'static str = "DefaultMaterial";

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiTextureOp {
    Multiply = 0x0,
    Add = 0x1,
    Subtract = 0x2,
    Divide = 0x3,
    SmoothAdd = 0x4,
    SignedAdd = 0x5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiTextureMapMode {
    Wrap = 0x0,
    Clamp = 0x1,
    Mirror = 0x2,
    Decal = 0x3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiTextureMapping {
    UV = 0x0,
    Sphere = 0x1,
    Cylinder = 0x2,
    Box = 0x3,
    Plane = 0x4,
    Other = 0x5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiTextureType {
    None = 0x0,
    Diffuse = 0x1,
    Specular = 0x2,
    Ambient = 0x3,
    Emissive = 0x4,
    Height = 0x5,
    Normals = 0x6,
    Shininess = 0x7,
    Opacity = 0x8,
    Displacement = 0x9,
    Lightmap = 0xA,
    Reflection = 0xB,
    Unknown = 0xC,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiShadingMode {
    Flat = 0x1,
    Gouraud = 0x2,
    Phong = 0x3,
    Blinn = 0x4,
    Toon = 0x5,
    OrenNayar = 0x6,
    Minnaert = 0x7,
    CookTorrance = 0x8,
    NoShading = 0x9,
    Fresnel = 0xA,
}

bitflags! {
    #[repr(C)]
    flags AiTextureFlags: c_uint {
        const AITEXTUREFLAG_INVERT = 0x1,
        const AITEXTUREFLAG_USE_ALPHA = 0x2,
        const AITEXTUREFLAG_IGNORE_ALPHA = 0x4
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiBlendMode {
    Default = 0x0,
    Additive = 0x1,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AiUVTransform {
    pub translation: AiVector2D,
    pub scaling: AiVector2D,
    pub rotation: c_float,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiPropertyTypeInfo {
    Float = 0x1,
    String = 0x3,
    Integer = 0x4,
    Buffer = 0x5,
}

#[repr(C)]
pub struct AiMaterialProperty {
    pub key: AiString,
    pub semantic: c_uint,
    pub index: c_uint,
    pub data_length: c_uint,
    pub property_type: AiPropertyTypeInfo,
    pub data: *mut c_char,
}

#[repr(C)]
pub struct AiMaterial {
    pub properties: *mut *mut AiMaterialProperty,
    pub num_properties: c_uint,
    pub num_allocated: c_uint,
}

extern {
    pub fn aiGetMaterialProperty(
        mat: *const AiMaterial,
        key: *const c_char,
        property_type: c_uint,
        index: c_uint,
        output: *const *const AiMaterialProperty) -> AiReturn;

    pub fn aiGetMaterialFloatArray(
        mat: *const AiMaterial,
        key: *const c_char,
        property_type: c_uint,
        index: c_uint,
        output: *mut c_float,
        max: *mut c_uint) -> AiReturn;

    pub fn aiGetMaterialIntegerArray(
        mat: *const AiMaterial,
        key: *const c_char,
        property_type: c_uint,
        index: c_uint,
        output: *mut c_int,
        max: *mut c_uint) -> AiReturn;

    pub fn aiGetMaterialColor(
        mat: *const AiMaterial,
        key: *const c_char,
        property_type: c_uint,
        index: c_uint,
        output: *mut AiColor4D) -> AiReturn;

    pub fn aiGetMaterialUVTransform(
        mat: *const AiMaterial,
        key: *const c_char,
        property_type: c_uint,
        index: c_uint,
        output: *mut AiUVTransform) -> AiReturn;

    pub fn aiGetMaterialString(
        mat: *const AiMaterial,
        key: *const c_char,
        property_type: c_uint,
        index: c_uint,
        output: *mut AiString) -> AiReturn;

    pub fn aiGetMaterialTextureCount(
        mat: *const AiMaterial,
        texture_type: AiTextureType) -> c_uint;

    pub fn aiGetMaterialTexture(
        mat: *const AiMaterial,
        texture_type: AiTextureType,
        index: c_uint,
        path: *mut AiString,
        mapping: *const AiTextureMapping,
        uv_index: *mut c_uint,
        blend: *mut c_float,
        op: *mut AiTextureOp,
        map_mode: *mut AiTextureMapMode,
        flags: *mut c_uint) -> AiReturn;
}
