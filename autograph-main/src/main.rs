#![feature(plugin, custom_attribute)]
#![plugin(autograph_codegen)]

#[macro_use]
extern crate autograph;
extern crate time;
extern crate pretty_env_logger;
extern crate glutin;
extern crate nalgebra;
extern crate alga;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate imgui;
extern crate imgui_sys;

mod imgui_glue;
mod main_loop;
mod renderer_passes;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::cmp::Ord;
use std::rc::Rc;
use glutin::GlContext;
use autograph::shader_preprocessor::*;
use autograph::shader_compiler::*;
use autograph::gfx;
use autograph::gl;
use autograph::gl::types::*;
use autograph::id_table::{ID,IDTable};
use autograph::scene_object::{SceneObject,SceneObjects,SceneMesh};
use autograph::scene_loader;
use autograph::cache;
use autograph::gfx::AsSlice;
use autograph::camera::*;
use autograph::framegraph::{FrameGraph, FrameGraphAllocator};
use nalgebra::*;

use main_loop::MainLoop;

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
    // Init logger
    pretty_env_logger::init().unwrap();
    // Init glutin, window with OpenGL context parameters
    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Autograph test")
        .with_dimensions(1280, 720);
    let context_builder = glutin::ContextBuilder::new()
        .with_gl_profile(glutin::GlProfile::Core)
        .with_gl_debug_flag(true)
        .with_vsync(true)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)));
    let window = glutin::GlWindow::new(window_builder, context_builder, &event_loop).unwrap();
    debug!("inner_size_points={:?}, inner_size_pixels={:?}", window.get_inner_size_points(), window.get_inner_size_pixels());
    // takes ownership of event_loop and window
    let main_loop = MainLoop::new(&window, &gfx::ContextConfig {
        max_frames_in_flight: 3
    });

    // initialize the imgui state
    let (mut imgui, mut imgui_renderer, mut imgui_mouse_state) = imgui_glue::init(
        main_loop.context(),
        main_loop.cache(),
        Some("data/fonts/iosevka-regular.ttf")
    );

    // create an upload buffer for uniforms
    let upload_buf = gfx::UploadBuffer::new(&main_loop.queue(), UPLOAD_BUFFER_SIZE);

    // load a scene!
    let mut ids = IDTable::new();
    let mut scene_objects = SceneObjects::new();
    let root_object_id = scene_loader::load_scene_file(
        Path::new("data/scenes/youmu/youmu.fbx"),
        &mut ids,
        main_loop.context(),
        main_loop.cache(),
        &mut scene_objects).unwrap();

    let mut camera_control = CameraControl::default();
    // allocations for the frame graph
    let mut fg_allocator = FrameGraphAllocator::new();

    // start main loop
    main_loop.run(
        //=================================================================
        // RENDER
        |frame, default_framebuffer, delta_s| {
            let mut running = true;
            // poll events
            event_loop.poll_events(|event| {
                // forward to imgui
                imgui_glue::handle_event(&mut imgui, &event, &mut imgui_mouse_state);
                // should close
                match event {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::Closed => running = false,
                        _ => ()
                    },
                    _ => ()
                }
            });

            scene_objects.calculate_transforms();

            // Clear the screen
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::Disable(gl::SCISSOR_TEST);
                gl::ClearColor(0.0, 1.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            // get framebuffer dimensions and aspect ratio
            let (width,height) = default_framebuffer.size();
            let (fwidth,fheight) = (width as f32, height as f32);
            let aspect_ratio = fwidth / fheight;

            // Create an IMGUI frame
            let ui = imgui.frame(
                window.get_inner_size_points().unwrap(),
                window.get_inner_size_pixels().unwrap(),
                delta_s);

            // setup camera parameters (center on root object)
            let root_bounds = scene_objects.get(root_object_id).unwrap().borrow().world_bounds;
            camera_control.set_aspect_ratio(aspect_ratio);
            let fovy = std::f32::consts::PI/4.0f32;
            camera_control.center_on_aabb(root_bounds, fovy);
            let cam = CameraParameters::from_camera(&camera_control.camera(), (0.0,0.0));

            // create a frame graph
            use renderer_passes::*;
            let mut fg = FrameGraph::new();
            let gbuffers = GBufferSetup::create(&mut fg, 640, 480);
            let after_scene = RenderScene::create(&mut fg, &scene_objects, RenderScene::Inputs {
                diffuse: gbuffers.diffuse,
                normals: gbuffers.normals,
                material_id: gbuffers.material_id,
                depth: gbuffers.depth
            });
            DeferredDebug::create(&mut fg, DeferredDebugBuffer::Diffuse, DeferredDebug::Inputs {
                diffuse: after_scene.diffuse,
                normals: after_scene.normals,
                material_id: after_scene.material_id,
                depth: after_scene.depth
            });

            // compile the frame graph
            let compiled_fg = fg.compile(&main_loop.queue(), &mut fg_allocator);
            // execute the frame graph
            compiled_fg.execute();


            // TODO: UBO alignment
            let cam_buffer = upload_buf.upload(frame, &cam, 256);

            for (id,obj) in scene_objects.iter() {
                // build draw command!
                let obj = obj.borrow();

                if let Some(ref sm) = obj.mesh {
                    //debug!("Render id {:?}", id);
                    let objparams = upload_buf.upload(frame, &ObjectParameters {
                        model_matrix: obj.world_transform.to_homogeneous(),
                        prev_model_matrix: obj.world_transform.to_homogeneous(),
                        object_id: id.idx as i32
                    }, 256);

                    /*gfx::DrawCommandBuilder::new(frame, default_framebuffer.clone(), pipeline.clone())
                        .with_vertex_buffer(0, &sm.mesh.vertex_buffer().as_slice())
                        .with_index_buffer(&sm.mesh.index_buffer().unwrap().as_slice())
                        .with_uniform_buffer(0, &cam_buffer)
                        .with_uniform_buffer(1, &objparams)
                        .command(&gfx::DrawIndexed {
                            first: 0,
                            count: sm.mesh.index_count(),
                            base_vertex: 0
                        });*/
                }
            }

            // UI test
            ui.window(im_str!("Hello world"))
                .size((300.0, 100.0), imgui::ImGuiSetCond_FirstUseEver)
                .build(|| {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("This...is...imgui-rs!"));
                    ui.separator();
                    let mouse_pos = ui.imgui().mouse_pos();
                    ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos.0, mouse_pos.1));
                });

            imgui_renderer.render(frame, default_framebuffer, &upload_buf, ui);
            running
        });
}

