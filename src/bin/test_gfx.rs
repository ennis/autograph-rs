extern crate autograph;

// The `vulkano` crate is the main crate that you must use to use Vulkan.
extern crate time;
extern crate pretty_env_logger;
extern crate glutin;
extern crate smallvec;
extern crate libc;
extern crate assimp;
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

use autograph::shader_preprocessor::preprocess_combined_shader_source;
use autograph::gfx;
use autograph::gl;
use autograph::id_table::{ID,IDTable};
use autograph::scene_object::{SceneObject,SceneObjects};

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
        // load combined shader source
        let mut src = String::new();
        let path = Path::new("data/shaders/DeferredGeometry.glsl");
        File::open(path).unwrap().read_to_string(&mut src).unwrap();
        // preprocess
        let (stages, results) = preprocess_combined_shader_source(&src, path, &[], &[]);
        // now build a pipeline
        gfx::GraphicsPipelineBuilder::new()
            .with_vertex_shader(gfx::Shader::compile(&results.vertex.unwrap(), gl::VERTEX_SHADER).map_err(|gfx::ShaderCompilationError(log)| println!("Shader compilation error: {}", log)).unwrap())
            .with_fragment_shader(gfx::Shader::compile(&results.fragment.unwrap(), gl::FRAGMENT_SHADER).map_err(|gfx::ShaderCompilationError(log)| println!("Shader compilation error: {}", log)).unwrap())
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
            .build(ctx).map_err(|e| match e {
                gfx::GraphicsPipelineBuildError::ProgramLinkError(ref log) => {
                    println!("Program link error: {}", log);
                }
            })
            .unwrap()
    };

    // load a scene!
    let mut ids = IDTable::new();
    let mut scene_objects = SceneObjects::new();


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