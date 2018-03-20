#![feature(plugin, custom_attribute)]
#![feature(const_fn, drop_types_in_const)]

#[macro_use]
extern crate autograph;
extern crate time;
extern crate pretty_env_logger;
extern crate glutin;
extern crate nalgebra;
extern crate alga;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate imgui;
extern crate imgui_sys;
#[macro_use]
extern crate autograph_derive;
extern crate image;

mod imgui_glue;
mod main_loop;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use glutin::GlContext;
use autograph::gfx;
use autograph::gl;
use autograph::gl::types::*;
use autograph::id_table::{IdTable, ID};
use autograph::scene_object::{SceneMesh, SceneObject, SceneObjects};
use autograph::scene_loader;
use autograph::cache;
use autograph::camera::*;
use autograph::framegraph::{FrameGraph, FrameGraphAllocator};
use nalgebra::*;

use autograph::gfx::DrawUtilsExt;
use autograph::gfx::glsl::GraphicsPipelineBuilderExt;

use main_loop::MainLoop;
use image::GenericImage;

const UPLOAD_BUFFER_SIZE: usize = 3 * 1024 * 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CameraParameters {
    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,
    viewproj_matrix: Matrix4<f32>,
    inverse_proj_matrix: Matrix4<f32>,
    prev_viewproj_matrix_velocity: Matrix4<f32>,
    viewproj_matrix_velocity: Matrix4<f32>,
    temporal_aa_offset: [f32; 2],
}

#[repr(C)]
#[derive(Copy,Clone,Debug)]
#[derive(VertexType)]
struct MyVertexType {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    texcoord: [f32; 2],
}

/*struct GpuVec2(Vector2<f32>);

unsafe impl gfx::VertexElementType for Vector2<f32>
{
    fn get_equivalent_type() -> gfx::Type {
        unimplemented!()
    }
}*/

#[derive(ShaderInterface)]
struct TestShaderInterface
{
    #[named_uniform(rename="transform")]
    matrix: [f32; 4],
    #[named_uniform]
    color: [f32; 4],
    #[texture_binding(index="0",rename="diffuse")]
    #[autobind(path="data/textures/background.png")]
    texture: gfx::RawTexture,
    #[vertex_buffer(index="0")]
    vertices: gfx::BufferSlice<[MyVertexType]>,
    #[index_buffer]
    indices: gfx::BufferSlice<[u32]>
}

impl CameraParameters {
    pub fn from_camera(cam: &Camera, temporal_aa_offset: (f32, f32)) -> CameraParameters {
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
            temporal_aa_offset: [0.0; 2], // TODO
        }
    }
}


fn main() {
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

    autograph::gl::load_with(|s| {
        let val = window.get_proc_address(s) as *const std::os::raw::c_void;
        println!("get_proc_address {} val {:?}", s, val);
        val
    });

    debug!("vertex layout = {:#?}", <MyVertexType as gfx::VertexType>::get_layout());
    debug!("textures = {:#?}", <TestShaderInterface as gfx::ShaderInterface>::get_description().get_texture_bindings());
    debug!("named uniforms = {:#?}", <TestShaderInterface as gfx::ShaderInterface>::get_description().get_named_uniforms());

    debug!(
        "inner_size_points={:?}, inner_size_pixels={:?}",
        window.get_inner_size_points(),
        window.get_inner_size_pixels()
    );
    // takes ownership of event_loop and window
    let main_loop = MainLoop::new(
        &window,
        &gfx::ContextConfig {
            max_frames_in_flight: 3,
        },
    );

    // load a test image
    let test_tex = (|| {
        let img = image::open("data/img/26459.png")?;
        let (width,height) = img.dimensions();
        let format = match img.color() {
            image::ColorType::RGB(8) => gfx::Format::R8G8B8_SRGB,
            image::ColorType::RGBA(8) => gfx::Format::R8G8B8A8_SRGB,
            _ => return Err(format_err!("Unsupported ColorType"))
        };
        let bytes: &[u8] = match img {
            image::DynamicImage::ImageLuma8(_) => return Err(format_err!("Unsupported ColorType")),
            image::DynamicImage::ImageLumaA8(_) => return Err(format_err!("Unsupported ColorType")),
            image::DynamicImage::ImageRgb8(ref rgb) => &*rgb,
            image::DynamicImage::ImageRgba8(ref rgba) => &*rgba
        };
        let texture_desc = gfx::TextureDesc {
            dimensions: gfx::TextureDimensions::Tex2D,
            format,
            width,
            height,
            depth: 1,
            sample_count: 0,
            mip_map_count: gfx::MipMaps::Count(1),
            options: gfx::TextureOptions::empty(),
        };

        Ok(gfx::RawTexture::with_pixels(main_loop.context(), &texture_desc, &bytes))
    })();

    // test pipeline
    let test_pipe = gfx::GraphicsPipelineBuilder::new()
        .with_glsl_file("data/shaders/textured_quad.glsl")
        .unwrap()
        .with_rasterizer_state(&gfx::RasterizerState {
            fill_mode: gl::FILL,
            ..Default::default()
        })
        .with_all_blend_states(&gfx::BlendState {
            enabled: true,
            mode_rgb: gl::FUNC_ADD,
            mode_alpha: gl::FUNC_ADD,
            func_src_rgb: gl::SRC_ALPHA,
            func_dst_rgb: gl::ONE_MINUS_SRC_ALPHA,
            func_src_alpha: gl::ONE,
            func_dst_alpha: gl::ZERO,
        })
        .build(main_loop.context()).unwrap();

    // initialize the imgui state
    let (mut imgui, mut imgui_renderer, mut imgui_mouse_state) = imgui_glue::init(
        main_loop.context(),
        main_loop.cache(),
        Some("data/fonts/iosevka-regular.ttf"),
    );

    // create an upload buffer for uniforms
    //let upload_buf = gfx::UploadBuffer::new(&main_loop.queue(), UPLOAD_BUFFER_SIZE);

    // load a scene!
    let mut ids = IdTable::new();
    let mut scene_objects = SceneObjects::new();
    let root_object_id = scene_loader::load_scene_file(
        Path::new("data/scenes/sponza/sponza.obj"),
        &mut ids,
        main_loop.context(),
        main_loop.cache(),
        &mut scene_objects,
    ).unwrap();

    let mut camera_control = CameraControl::default();
    // allocations for the frame graph
    let mut fg_allocator = FrameGraphAllocator::new();
    let mut bgcolor = [0f32; 3];

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
                gl::ClearColor(bgcolor[0], bgcolor[1], bgcolor[2], 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            // get framebuffer dimensions and aspect ratio
            let (width,height) = default_framebuffer.size();
            let (fwidth,fheight) = (width as f32, height as f32);
            let aspect_ratio = fwidth / fheight;

            // Create an IMGUI frame
            let ui = imgui.frame(
                window.get_inner_size().unwrap(),
                window.get_inner_size().unwrap(),
                delta_s);

            // setup camera parameters (center on root object)
            let root_bounds = scene_objects.get(root_object_id).unwrap().borrow().world_bounds;
            camera_control.set_aspect_ratio(aspect_ratio);
            let fovy = std::f32::consts::PI/4.0f32;
            camera_control.center_on_aabb(root_bounds, fovy);
            let camera = camera_control.camera();

            // create a frame graph
            let mut fg = FrameGraph::new();

            // new node
            let ectx = fg.finalize(main_loop.context(), &mut fg_allocator).unwrap();
            ectx.execute(frame);

            // texture test
            if let Ok(ref tex) = test_tex {
                frame.draw_quad(default_framebuffer, &test_pipe, (-1.0f32, 1.0f32, -1.0f32, 1.0f32))
                     .with_texture(0, &tex, &gfx::LINEAR_WRAP_SAMPLER);
            }

            // UI test
            ui.window(im_str!("Hello world"))
                .size((300.0, 100.0), imgui::ImGuiCond::FirstUseEver)
                .build(|| {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("This...is...imgui-rs!"));
                    ui.separator();
                    let mouse_pos = ui.imgui().mouse_pos();
                    ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos.0, mouse_pos.1));
                    ui.color_picker(im_str!("Background color"), &mut bgcolor).build();
                });

            ui.main_menu_bar(|| {
                ui.menu(im_str!("Engine")).build(|| {
                    ui.menu_item(im_str!("Take screenshot")).build();
                });
                ui.text(im_str!("Frame time: {}", delta_s));
            });

            imgui_renderer.render(frame, default_framebuffer, ui);
            running
        });
}
