use gfx;
use std::path::Path;
use std::fs::File;
use shader_preprocessor::*;
use gl;
use gl::types::*;
use std::io::Read;

pub struct CompiledShaders {
    pub vertex: gfx::Shader,
    pub fragment: gfx::Shader,
    pub geometry: Option<gfx::Shader>,
    pub tess_control: Option<gfx::Shader>,
    pub tess_eval: Option<gfx::Shader>,
    pub input_layout: Vec<gfx::VertexAttribute>,
    pub primitive_topology: GLenum
}

pub fn compile_shaders_from_combined_source(src_path: &Path) -> Result<CompiledShaders, String>
{
    // load combined shader source
    let mut src = String::new();
    File::open(src_path).unwrap().read_to_string(&mut src).unwrap();
    // preprocess
    let (stages, pp) = preprocess_combined_shader_source(&src, src_path, &[], &[]);

    // try to compile shaders
    let print_error_log = |log: &str, stage| {
        error!("====================================================================");
        error!("Shader compilation error ({:?}) | stage: {:?}", src_path, stage);
        error!("{}\n", log);
    };

    // Compile shaders
    let vertex = gfx::Shader::compile(&pp.vertex.unwrap(), gl::VERTEX_SHADER).map_err(|log| { print_error_log(&log, PS_VERTEX); log } )?;
    let fragment = gfx::Shader::compile(&pp.fragment.unwrap(), gl::FRAGMENT_SHADER).map_err(|log| { print_error_log(&log, PS_FRAGMENT); log } )?;

    let geometry = if let Some(ref geometry) = pp.geometry {
        Some(gfx::Shader::compile(&geometry, gl::GEOMETRY_SHADER).map_err(|log| { print_error_log(&log, PS_GEOMETRY); log } )?)
    } else {
        None
    };

    let tess_control = if let Some(ref tess_control) = pp.tess_control {
        Some(gfx::Shader::compile(&tess_control, gl::TESS_CONTROL_SHADER).map_err(|log| { print_error_log(&log, PS_TESS_CONTROL); log } )?)
    } else {
        None
    };

    let tess_eval = if let Some(ref tess_eval) = pp.tess_eval {
        Some(gfx::Shader::compile(&tess_eval, gl::TESS_EVALUATION_SHADER).map_err(|log| { print_error_log(&log, PS_TESS_EVAL); log } )?)
    } else {
        None
    };

    // Specify layout
    Ok(
        CompiledShaders {
            vertex, fragment, geometry, tess_control, tess_eval,
            input_layout: pp.input_layout.ok_or("Missing input layout in combined shader source".to_owned())?,
            primitive_topology: pp.primitive_topology.ok_or("Missing primitive topology in combined shader source".to_owned())?
        }
    )
}
