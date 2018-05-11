#[macro_use]
extern crate autograph;
#[macro_use]
extern crate autograph_derive;

#[macro_use]
mod common;
use autograph::gfx;

#[repr(C)]
#[derive(Copy, Clone, BufferLayout)]
struct CameraParams {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 3]; 3],
    viewproj_matrix: [[f32; 4]; 4],
    inverse_proj_matrix: [[f32; 4]; 4],
    prev_viewproj_matrix_velocity: [[f32; 4]; 4],
    viewproj_matrix_velocity: [[f32; 4]; 4],
    temporal_aa_offset: [f32; 2],
}

#[derive(ShaderInterface)]
struct Interface0 {
    #[uniform_constant(index = "0")]
    a: f32,
    #[uniform_constant(index = "1")]
    b: f32,
    #[texture_binding(index = "0")]
    tex: gfx::SampledTexture2D,
    #[uniform_buffer(index = "0")]
    camera_params: gfx::BufferSlice<CameraParams>,
}

#[test]
fn test_shader_interface_basic() {
    load_pipeline_and_check_interface::<Interface0>(make_interface_test_shader! { r#"
layout(location=0) uniform float A;
layout(binding=0) uniform sampler2D tex;

layout(binding=0,std140) uniform U {
        mat4 viewMatrix;
        mat3 projMatrix;
        mat4 viewProjMatrix;
        mat4 invViewProjMatrix;
        mat4 prevViewProjMatrixVelocity;
        mat4 viewProjMatrixVelocity;
        ivec2 temporalAAOffset;
};
"# });
}
