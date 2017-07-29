extern crate autograph;

// The `vulkano` crate is the main crate that you must use to use Vulkan.
extern crate time;
extern crate pretty_env_logger;
extern crate glutin;
extern crate smallvec;
extern crate libc;
extern crate nalgebra;
extern crate alga;
extern crate regex;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate itertools;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::collections::BTreeSet;
use std::cmp::Ord;

use glutin::GlContext;

use autograph::shader_preprocessor::*;
use autograph::gfx;
use autograph::gl;
use autograph::gl::types::*;
use autograph::id_table::{ID,IDTable};
use autograph::scene_object::{SceneObject,SceneObjects};
use autograph::scene_loader;
use autograph::rc_cache;

struct CompiledShaders {
    vertex: gfx::Shader,
    fragment: gfx::Shader,
    geometry: Option<gfx::Shader>,
    tess_control: Option<gfx::Shader>,
    tess_eval: Option<gfx::Shader>,
    input_layout: Vec<gfx::VertexAttribute>,
    primitive_topology: GLenum
}

fn compile_shaders_from_combined_source(src_path: &Path) -> Result<CompiledShaders, String>
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

fn main()
{
    pretty_env_logger::init().unwrap();

    info!("Testing log output");

    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Autograph test")
        .with_dimensions(640, 480);
    let context_builder = glutin::ContextBuilder::new()
        .with_gl_profile(glutin::GlProfile::Core)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)));
    let window = glutin::GlWindow::new(window_builder, context_builder, &event_loop).unwrap();

    unsafe {
        window.make_current()
    }.unwrap();

    unsafe {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    // for posterity
    println!("Hello, world!");
    let mut running = true;

    // create a context
    let ctx = gfx::Context::new(&gfx::ContextConfig {
        default_upload_buffer_size: 3 * 1024 * 1024,
        max_frames_in_flight: 3
    });

    // create a texture bound to this context
    let tex = gfx::Texture::new(ctx.clone(), &gfx::TextureDesc {
        dimensions: gfx::TextureDimensions::Tex2D,
        format: gfx::TextureFormat::R8G8B8A8_UNORM,
        width: 640,
        height: 480,
        ..Default::default()
    });

    // load a pipeline!
    let pipeline = {
        // now build a pipeline
        let compiled_shaders = compile_shaders_from_combined_source(Path::new("data/shaders/DeferredGeometry.glsl")).unwrap();
        println!("layout: {:#?}", &compiled_shaders.input_layout);
        gfx::GraphicsPipelineBuilder::new()
            .with_vertex_shader(compiled_shaders.vertex)
            .with_fragment_shader(compiled_shaders.fragment)
            .with_geometry_shader(compiled_shaders.geometry)
            .with_tess_eval_shader(compiled_shaders.tess_eval)
            .with_tess_control_shader(compiled_shaders.tess_control)
            .with_primitive_topology(compiled_shaders.primitive_topology)
            .with_rasterizer_state(&gfx::RasterizerState{
                fill_mode: gl::LINE,
                .. Default::default()
            })
            .with_input_layout(&[
                gfx::VertexAttribute {
                    slot: 0,
                    ty: gl::FLOAT,
                    size: 3,
                    relative_offset: 0,
                    normalized: false
                },
                gfx::VertexAttribute {
                    slot: 0,
                    ty: gl::FLOAT,
                    size: 3,
                    relative_offset: 12,
                    normalized: false
                },
                gfx::VertexAttribute {
                    slot: 0,
                    ty: gl::FLOAT,
                    size: 3,
                    relative_offset: 24,
                    normalized: false
                },
                gfx::VertexAttribute {
                    slot: 0,
                    ty: gl::FLOAT,
                    size: 2,
                    relative_offset: 36,
                    normalized: false
                }])
            .build(ctx.clone()).map_err(|e| match e {
                gfx::GraphicsPipelineBuildError::ProgramLinkError(ref log) => {
                    println!("Program link error: {}", log);
                }
            })
            .unwrap()
    };

    // load a scene!
    let mut ids = IDTable::new();
    let mut scene_objects = SceneObjects::new();
    let mut cache = rc_cache::Cache::new();
    scene_loader::load_scene_file(Path::new("data/scenes/sponza/sponza.obj"), &mut ids, ctx.clone(), &cache, &mut scene_objects);

    println!("Pipeline: {:#?}", pipeline);

    // draw macro with dynamic pipelines
    // <binding-type> <name> = initializer
    // OR: <binding-type> <index> = initializer
    // OR: <binding-type>: initializer

    /*gfx_draw!(
        target:                     fbo,
        command:                    DrawArrays { ..unimplemented!() },
        uniform uPrevModelMatrix:   unimplemented!(),
        uniform uObjectID:          unimplemented!(),
        uniform_buffer[0]:          unimplemented!(),
        sampled_texture[0]:         (tex, sampler),
    );*/

    /*gfx_draw!(
        target:         fbo,
        command:        DrawArrays { ... },
        pipeline:       DynamicPipeline,
        vertex_buffer(0):  ,
        index_buffer:   ,
        uniform Name = "...",
        uniform_buffer Struct = "...",
        ssbo Name = <some slice>,
    );*/

    while running {
        // poll events
        event_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event: glutin::WindowEvent::Closed, .. } => {
                    running = false;
                }
                _ => ()
            }
        });

        // once all events in the queue are dispatched, render stuff
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // swap buffers
        window.swap_buffers().unwrap();
    }
}