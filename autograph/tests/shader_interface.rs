#[macro_use]
extern crate autograph;
#[macro_use]
extern crate autograph_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

use autograph::gfx;
use autograph::gfx::glsl::interface::{verify_spirv_interface, ShaderInterfaceVerificationError};
use autograph::gfx::glsl::{compile_glsl_to_spirv, preprocess_combined_shader_source,
                           SourceWithFileName, SpirvModules};
use autograph::gfx::shader_interface::{ShaderInterface, ShaderInterfaceDesc};
use autograph::gfx::GraphicsShaderPipeline;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn load_spv_modules(src: &str) -> SpirvModules {
    let (_, pp) = preprocess_combined_shader_source(src, "<internal>", &[], &[]);
    let src_path_str = "<internal>";
    let spv_modules = compile_glsl_to_spirv(
        SourceWithFileName {
            source: pp.vertex.as_ref().unwrap(),
            file_name: &src_path_str,
        },
        SourceWithFileName {
            source: pp.fragment.as_ref().unwrap(),
            file_name: &src_path_str,
        },
        pp.geometry.as_ref().map(|geom| SourceWithFileName {
            source: geom,
            file_name: &src_path_str,
        }),
        pp.tess_control
            .as_ref()
            .map(|tess_control| SourceWithFileName {
                source: tess_control,
                file_name: &src_path_str,
            }),
        pp.tess_eval.as_ref().map(|tess_eval| SourceWithFileName {
            source: tess_eval,
            file_name: &src_path_str,
        }),
    ).unwrap();
    spv_modules
}

fn dump_error(error: &failure::Error) {
    let mut fail = error.cause();
    eprintln!("error: {}", fail);
    while let Some(cause) = fail.cause() {
        eprintln!("Caused by: {}", cause);
        fail = cause;
    }
}

fn load_pipeline_and_check_interface<I: ShaderInterface>(src: &str) {
    let spv = load_spv_modules(src);
    let desc = <I as ShaderInterface>::get_description();
    let result = verify_spirv_interface(
        desc,
        spv.vs.as_ref(),
        spv.fs.as_ref(),
        spv.gs.as_ref().map(|v| v.as_ref()),
        spv.tcs.as_ref().map(|v| v.as_ref()),
        spv.tes.as_ref().map(|v| v.as_ref()),
    );
    if let Err(ShaderInterfaceVerificationError(ref errors)) = result {
        for err in errors.iter() {
            dump_error(err);
            eprintln!();
        }
        panic!()
    }
}

macro_rules! shader_skeleton {
    ($src:expr) => {
        concat!(
            r#"#version 450
#pragma stages(vertex,fragment)
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable"#,
            $src,
            r#"
#ifdef _VERTEX_
// visible to this stage only
layout(location=1) uniform float b;
void main() {
  gl_Position = vec4(0.0);
}
#endif
#ifdef _FRAGMENT_
layout(location = 0) out vec4 color;
void main() {
    color = vec4(0.0);
}
#endif
"#
        );
    };
}

#[repr(C)]
#[derive(Copy,Clone,BufferLayout)]
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
fn test_stuff() {
    load_pipeline_and_check_interface::<Interface0>(shader_skeleton! { r#"
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
