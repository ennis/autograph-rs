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
use autograph::gfx::glsl::{compile_glsl_to_spirv, load_combined_shader_source, SourceWithFileName,
                           SpirvModules};
use autograph::gfx::shader_interface::{ShaderInterface, ShaderInterfaceDesc};
use autograph::gfx::GraphicsShaderPipeline;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[repr(C)]
#[derive(BufferInterface)]
struct CameraParams {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
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
    tex: gfx::Texture2D,
    #[uniform_buffer(index = "0")]
    camera_params: CameraParams
}

fn load_spv_modules<P: AsRef<Path>>(p: P) -> SpirvModules {
    let pp = load_combined_shader_source(p.as_ref()).unwrap();
    let src_path_str = p.as_ref().to_str().unwrap();
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

fn load_pipeline_and_check_interface<I: ShaderInterface, P: AsRef<Path>>(p: P) {
    let spv = load_spv_modules(p.as_ref());
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

#[test]
fn test_stuff() {
    load_pipeline_and_check_interface::<Interface0, _>("tests/interface/simple.glsl");
}
