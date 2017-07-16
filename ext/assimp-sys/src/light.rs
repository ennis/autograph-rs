use std::os::raw::c_float;

use types::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiLightSourceType {
    Undefined = 0x0,
    Directional = 0x1,
    Point = 0x2,
    Spot = 0x3,
    Ambient = 0x4,
    Area = 0x5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiLight {
    pub name: AiString,
    pub light_type: AiLightSourceType,
    pub position: AiVector3D,
    pub direction: AiVector3D,
    pub up: AiVector3D,
    pub attenuation_constant: c_float,
    pub attenuation_linear: c_float,
    pub attenuation_quadratic: c_float,
    pub color_diffuse: AiColor3D,
    pub color_specular: AiColor3D,
    pub color_ambient: AiColor3D,
    pub angle_inner_cone: c_float,
    pub angle_outer_cone: c_float,
    pub size: AiVector2D,
}
