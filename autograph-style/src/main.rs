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

/// Application state
struct StyleApp
{
    ctx: gfx::Context,
    obj_path: String,
    texture_image_path: String,
    // Reloadable
    pipelines: Pipelines,
    // Resizable
    resources: Resources
    //
    idtable:
}

struct Pipelines
{
    gbuffers: gfx::GraphicsPipeline,
    compose: gfx::GraphicsPipeline,
}

struct Resources
{
    depth: gfx::RawTexture,
    normals: gfx::RawTexture,
    normals_and_depth: gfx::RawTexture,
    object_space_pos: gfx::RawTexture,
    object_id: gfx::RawTexture,
    tangents: gfx::RawTexture,
    diffuse: gfx::RawTexture,
    gbuffers_fbo: gfx::Framebuffer,
}

const DEFAULT_BLEND_STATE: gfx::BlendState = gfx::BlendState {
    enabled: true,
    mode_rgb: gl::FUNC_ADD,
    mode_alpha: gl::FUNC_ADD,
    func_src_rgb: gl::SRC_ALPHA,
    func_dst_rgb: gl::ONE_MINUS_SRC_ALPHA,
    func_src_alpha: gl::ONE,
    func_dst_alpha: gl::ZERO,
};

const DEFAULT_RASTERIZER_STATE: gfx::RasterizerState = gfx::RasterizerState {
    fill_mode: gl::FILL,
    ..Default::default()
};

fn reload_pipelines(ctx: &gfx::Context) -> Pipelines
{
    // GBuffers
    let gbuffers = gfx::GraphicsPipelineBuilder::new()
        .with_glsl_file("data/style/gbuffers.glsl")
        .unwrap()
        .with_rasterizer_state(&DEFAULT_RASTERIZER_STATE)
        .with_all_blend_states(&DEFAULT_BLEND_STATE)
        .build(main_loop.context()).unwrap();

    // compositing node
    let compose = gfx::GraphicsPipelineBuilder::new()
        .with_glsl_file("data/style/compose.glsl")
        .unwrap()
        .with_rasterizer_state(&DEFAULT_RASTERIZER_STATE)
        .with_all_blend_states(&DEFAULT_BLEND_STATE)
        .build(main_loop.context()).unwrap();

    Pipelines { gbuffers, compose }
}

fn make_tex2d(ctx: &gfx::Context, width: u32, height: u32, format: gfx::Format) -> gfx::RawTexture
{
    let desc = gfx::TextureDesc {
        dimensions: gfx::TextureDimensions::Tex2D,
        format,
        width,
        height,
        depth: 1,
        sample_count: 0,
        mip_map_count: gfx::MipMaps::Count(1),
        options: gfx::TextureOptions::empty(),
    };

    gfx::RawTexture::new(ctx, &texture_desc)
}

fn alloc_resources(ctx: &gfx::Context, w: u32, h: u32) -> Resources
{
    // don't take any chances with the format: R32Fx4 for everything, like gratin
    let depth = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let normals = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let normals_and_depth = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let object_space_pos = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let object_id = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let tangents = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let bitangents = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);
    let diffuse = make_tex2d(ctx, w, h, gfx::Format::R32G32B32A32_SFLOAT);

    let gbuffers_fbo = gfx::FramebufferBuilder::new(ctx)
        .attach_texture(0,depth)
        .attach_texture(1,normals)
        .attach_texture(2,tangents)
        .attach_texture(3,bitangents)
        .attach_texture(4,object_space_pos)
        .attach_texture(5,object_id)
        .attach_texture(6,diffuse);

    Resources {
        depth,
        normals,
        normals_and_depth,
        object_space_pos,
        object_id,
        tangents,
        diffuse,
        gbuffers_fbo,
    }
}

impl StyleApp
{
    fn reload(&mut self) {
        self.pipelines = reload_pipelines(self.ctx);
    }

    fn render(&mut self) {

    }

    fn render_gui(&mut self, ui: Ui) {
        // TODO
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
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 6)));
    let window = glutin::GlWindow::new(window_builder, context_builder, &event_loop).unwrap();

    autograph::gl::load_with(|s| {
        let val = window.get_proc_address(s) as *const std::os::raw::c_void;
        println!("get_proc_address {} val {:?}", s, val);
        val
    });

    // initialize the imgui state
    let (mut imgui, mut imgui_renderer, mut imgui_mouse_state) = imgui_glue::init(
        main_loop.context(),
        main_loop.cache(),
        Some("data/fonts/iosevka-regular.ttf"),
    );

    // load a scene
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
