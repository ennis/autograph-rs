#![feature(plugin, custom_attribute)]
extern crate flame;
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
use std::rc::Rc;

use glutin::GlContext;

use autograph::shader_preprocessor::*;
use autograph::gfx;
use autograph::gl;
use autograph::gl::types::*;
use autograph::id_table::{ID,IDTable};
use autograph::scene_object::{SceneObject,SceneObjects,SceneMesh};
use autograph::scene_loader;
use autograph::rc_cache;
use autograph::gfx::AsSlice;
use autograph::camera::*;

use nalgebra::*;

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

const UPLOAD_BUFFER_SIZE: usize = 3*1024*1024;

#[repr(C)]
#[derive(Copy,Clone,Debug)]
struct CameraParameters {
    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,
    viewproj_matrix: Matrix4<f32>,
    inverse_proj_matrix: Matrix4<f32>,
    prev_viewproj_matrix_velocity: Matrix4<f32>,
    viewproj_matrix_velocity: Matrix4<f32>,
    temporal_aa_offset: [f32; 2]
}

impl CameraParameters
{
    pub fn from_camera(cam: &Camera, temporal_aa_offset: (f32,f32)) -> CameraParameters {
        let view_matrix = cam.view.to_homogeneous();
        let proj_matrix = cam.projection.unwrap();
        let viewproj_matrix = proj_matrix * view_matrix;
        let inverse_proj_matrix = cam.projection.inverse();

        CameraParameters {
            view_matrix,
            proj_matrix,
            viewproj_matrix,
            inverse_proj_matrix,
            viewproj_matrix_velocity: viewproj_matrix,
            prev_viewproj_matrix_velocity: viewproj_matrix,
            temporal_aa_offset: [0.0;2] // TODO
        }
    }
}

// Per-object parameters
#[repr(C)]
#[derive(Copy,Clone,Debug)]
struct ObjectParameters {
    model_matrix: Matrix4<f32>,
    prev_model_matrix: Matrix4<f32>,
    object_id: i32
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
        .with_gl_debug_flag(true)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)));
    let window = glutin::GlWindow::new(window_builder, context_builder, &event_loop).unwrap();

    unsafe {
        window.make_current()
    }.unwrap();

    unsafe {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    }

    // for posterity
    println!("Hello, world!");
    let mut running = true;

    // create a context
    let ctx = gfx::Context::new(&gfx::ContextConfig {
        default_upload_buffer_size: UPLOAD_BUFFER_SIZE,
        max_frames_in_flight: 3
    });
    // create a queue
    let queue = gfx::FrameQueue::new(ctx.clone());

    // create an upload buffer for uniforms
    let upload_buf = gfx::UploadBuffer::new(&queue, UPLOAD_BUFFER_SIZE);

    // load a pipeline!
    let pipeline = {
        // now build a pipeline
        let compiled_shaders = compile_shaders_from_combined_source(Path::new("data/shaders/DeferredGeometry.glsl")).unwrap();
        Rc::new(gfx::GraphicsPipelineBuilder::new()
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
            .unwrap())
    };

    // load a scene!
    let mut ids = IDTable::new();
    let mut scene_objects = SceneObjects::new();
    let mut cache = rc_cache::Cache::new();
    let rootobjid = scene_loader::load_scene_file(Path::new("data/scenes/sponza/sponza.obj"), &mut ids, ctx.clone(), &cache, &mut scene_objects).unwrap();
    let mut camera_control = CameraControl::default();

    let mut update = || {
    };

    let mut render = |frame: &mut gfx::Frame| {
        scene_objects.calculate_transforms();

        // Clear the screen
        unsafe {
            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let framebuffer = Rc::new(gfx::Framebuffer::from_gl_window(ctx.clone(), &window));
        let (width,height) = framebuffer.size();
        let (fwidth,fheight) = (width as f32, height as f32);
        let aspect_ratio = fwidth / fheight;
        let fovy = std::f32::consts::PI/4.0f32;

        // setup camera parameters (center on root object)
        let root_bounds = scene_objects.get(rootobjid).unwrap().borrow().world_bounds;
        camera_control.set_aspect_ratio(aspect_ratio);
        camera_control.center_on_aabb(root_bounds, fovy);
        let cam = CameraParameters::from_camera(&camera_control.camera(), (0.0,0.0));

        // TODO: UBO alignment
        let cam_buffer = upload_buf.upload(frame, &cam, 256);

        for (id,obj) in scene_objects.iter() {
            // build draw command!
            let obj = obj.borrow();

            if let Some(ref sm) = obj.mesh {
                debug!("Render id {:?}", id);

                let objparams = upload_buf.upload(frame, &ObjectParameters {
                    model_matrix: obj.world_transform.to_homogeneous(),
                    prev_model_matrix: obj.world_transform.to_homogeneous(),
                    object_id: id.idx as i32
                }, 256);

                gfx::DrawCommandBuilder::new(frame, framebuffer.clone(), pipeline.clone())
                    .with_vertex_buffer(0, &sm.mesh.vertex_buffer().as_slice())
                    .with_index_buffer(&sm.mesh.index_buffer().unwrap().as_slice())
                    .with_uniform_buffer(0, &cam_buffer)
                    .with_uniform_buffer(1, &objparams)
                    .command(&gfx::DrawIndexed {
                        first: 0,
                        count: sm.mesh.index_count(),
                        base_vertex: 0
                    });
            }
        }
    };

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

        // create the frame
        let mut frame = queue.new_frame();

        // update scene
        update();
        // once all events in the queue are dispatched, render stuff
        render(&mut frame);
        // submit frame
        frame.submit();

        // swap buffers
        window.swap_buffers().unwrap();
    }

    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
}