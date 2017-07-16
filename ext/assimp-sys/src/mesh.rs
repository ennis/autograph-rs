use std::os::raw::{c_float, c_uint};

use types::*;

pub const AI_MAX_FACE_INDICES: usize = 0x7fff;
pub const AI_MAX_BONE_WEIGHTS: usize = 0x7fffffff;
pub const AI_MAX_VERTICES: usize = 0x7fffffff;
pub const AI_MAX_FACES: usize = 0x7fffffff;
pub const AI_MAX_NUMBER_OF_COLOR_SETS: usize = 0x8;
pub const AI_MAX_NUMBER_OF_TEXTURECOORDS: usize = 0x8;

#[repr(C)]
pub struct AiFace {
    pub num_indices: c_uint,
    pub indices: *mut c_uint,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiVertexWeight {
    pub vertex_id: c_uint,
    pub weight: c_float,
}

#[repr(C)]
pub struct AiBone {
    pub name: AiString,
    pub num_weights: c_uint,
    pub weights: *mut AiVertexWeight,
    pub offset_matrix: AiMatrix4x4,
}

bitflags! {
    #[repr(C)]
    flags AiPrimitiveType: c_uint {
        const AIPRIMITIVETYPE_POINT = 0x1,
        const AIPRIMITIVETYPE_LINE = 0x2,
        const AIPRIMITIVETYPE_TRIANGLE = 0x4,
        const AIPRIMITIVETYPE_POLYGON = 0x8
    }
}

#[repr(C)]
pub struct AiAnimMesh {
    pub vertices: *mut AiVector3D,
    pub normals: *mut AiVector3D,
    pub tangents: *mut AiVector3D,
    pub bitangents: *mut AiVector3D,
    pub colors: [*mut AiColor4D; AI_MAX_NUMBER_OF_COLOR_SETS],
    pub texture_coords: [*mut AiVector3D; AI_MAX_NUMBER_OF_TEXTURECOORDS],
    pub num_vertices: c_uint,
}

impl AiAnimMesh {
    pub fn has_positions(&self) -> bool {
        !self.vertices.is_null()
    }
    pub fn has_normals(&self) -> bool {
        !self.normals.is_null()
    }
    pub fn has_tangents_and_bitangents(&self) -> bool {
        !self.tangents.is_null()
    }
    pub fn has_vertex_colors(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_COLOR_SETS && !self.colors[index].is_null()
    }
    pub fn has_texture_coords(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_TEXTURECOORDS && !self.texture_coords[index].is_null()
    }
}

#[repr(C)]
pub struct AiMesh {
    pub primitive_types: c_uint,
    pub num_vertices: c_uint,
    pub num_faces: c_uint,
    pub vertices: *mut AiVector3D,
    pub normals: *mut AiVector3D,
    pub tangents: *mut AiVector3D,
    pub bitangents: *mut AiVector3D,
    pub colors: [*mut AiColor4D; AI_MAX_NUMBER_OF_COLOR_SETS],
    pub texture_coords: [*mut AiVector3D; AI_MAX_NUMBER_OF_TEXTURECOORDS],
    pub num_uv_components: [c_uint; AI_MAX_NUMBER_OF_TEXTURECOORDS],
    pub faces: *mut AiFace,
    pub num_bones: c_uint,
    pub bones: *mut *mut AiBone,
    pub material_index: c_uint,
    pub name: AiString,
    pub num_anim_meshes: c_uint,
    pub anim_meshes: *mut *mut AiAnimMesh,
}

impl AiMesh {
    pub fn has_positions(&self) -> bool {
        !self.vertices.is_null() && self.num_vertices > 0
    }
    pub fn has_faces(&self) -> bool {
        !self.faces.is_null() && self.num_faces > 0
    }
    pub fn has_normals(&self) -> bool {
        !self.normals.is_null() && self.num_vertices > 0
    }
    pub fn has_tangents_and_bitangents(&self) -> bool {
        !self.tangents.is_null() && self.num_vertices > 0
    }
    pub fn has_vertex_colors(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_COLOR_SETS && !self.colors[index].is_null() &&
        self.num_vertices > 0
    }
    pub fn has_texture_coords(&self, index: usize) -> bool {
        index < AI_MAX_NUMBER_OF_TEXTURECOORDS && !self.texture_coords[index].is_null() &&
        self.num_vertices > 0
    }
    pub fn get_num_uv_channels(&self) -> usize {
        let mut n = 0;
        while n < AI_MAX_NUMBER_OF_TEXTURECOORDS && !self.texture_coords[n].is_null() {
            n += 1;
        }
        n
    }
    pub fn get_num_color_channels(&self) -> usize {
        let mut n = 0;
        while n < AI_MAX_NUMBER_OF_COLOR_SETS && !self.colors[n].is_null() {
            n += 1;
        }
        n
    }
    pub fn has_bones(&self) -> bool {
        !self.bones.is_null() && self.num_bones > 0
    }
}
