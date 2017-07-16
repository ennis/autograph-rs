use std::os::raw::{c_uint, c_void};

use anim::*;
use camera::*;
use light::*;
use material::*;
use mesh::*;
use metadata::*;
use texture::*;
use types::*;

#[repr(C)]
pub struct AiNode {
    pub name: AiString,
    pub transformation: AiMatrix4x4,
    pub parent: *mut AiNode,
    pub num_children: c_uint,
    pub children: *mut *mut AiNode,
    pub num_meshes: c_uint,
    pub meshes: *mut c_uint,
    pub metadata: *mut AiMetadata,
}

bitflags! {
    #[repr(C)]
    flags AiSceneFlags : c_uint {
        const AI_SCENE_FLAGS_INCOMPLETE = 0x1,
        const AI_SCENE_FLAGS_VALIDATED = 0x2,
        const AI_SCENE_FLAGS_VALIDATION_WARNING = 0x4,
        const AI_SCENE_FLAGS_NON_VERBOSE_FORMAT = 0x8,
        const AI_SCENE_FLAGS_TERRAIN = 0x10
    }
}

#[repr(C)]
pub struct AiScene {
    pub flags: AiSceneFlags,
    pub root_node: *mut AiNode,
    pub num_meshes: c_uint,
    pub meshes: *mut *mut AiMesh,
    pub num_materials: c_uint,
    pub materials: *mut *mut AiMaterial,
    pub num_animations: c_uint,
    pub animations: *mut *mut AiAnimation,
    pub num_textures: c_uint,
    pub textures: *mut *mut AiTexture,
    pub num_lights: c_uint,
    pub lights: *mut *mut AiLight,
    pub num_cameras: c_uint,
    pub cameras: *mut *mut AiCamera,
    private: *const c_void,
}

impl AiScene {
    pub fn has_meshes(&self) -> bool {
        !self.meshes.is_null() && self.num_meshes > 0
    }
    pub fn has_materials(&self) -> bool {
        !self.materials.is_null() && self.num_materials > 0
    }
    pub fn has_lights(&self) -> bool {
        !self.lights.is_null() && self.num_lights > 0
    }
    pub fn has_textures(&self) -> bool {
        !self.textures.is_null() && self.num_textures > 0
    }
    pub fn has_cameras(&self) -> bool {
        !self.cameras.is_null() && self.num_cameras > 0
    }
    pub fn has_animations(&self) -> bool {
        !self.animations.is_null() && self.num_animations > 0
    }
}
