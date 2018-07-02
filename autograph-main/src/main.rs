#![feature(plugin, custom_attribute)]
#![feature(const_fn)]

#[macro_use]
extern crate autograph;
extern crate alga;
extern crate glutin;
extern crate nalgebra;
extern crate pretty_env_logger;
extern crate time;
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
extern crate nfd;

mod imgui_glue;
mod main_loop;

use autograph::cache::Cache;
use autograph::camera::*;
use autograph::framegraph::{FrameGraph, FrameGraphAllocator};
use autograph::gfx;
use autograph::gl;
use autograph::gl::types::*;
use autograph::id_table::{IdTable, ID};
use autograph::rect_transform::*;
use autograph::scene_loader;
use autograph::scene_object::{SceneMesh, SceneObject, SceneObjects};
use glutin::GlContext;
use nalgebra::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use autograph::gfx::glsl::GraphicsPipelineBuilderExt;
use autograph::gfx::DrawUtilsExt;
use autograph::gfx::GraphicsPipelineBuilder;

use failure::Error;
use image::GenericImage;
use main_loop::MainLoop;

const UPLOAD_BUFFER_SIZE: usize = 3 * 1024 * 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferLayout)]
struct CameraParameters {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    viewproj_matrix: [[f32; 4]; 4],
    inverse_proj_matrix: [[f32; 4]; 4],
    prev_viewproj_matrix_velocity: [[f32; 4]; 4],
    viewproj_matrix_velocity: [[f32; 4]; 4],
    temporal_aa_offset: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferLayout)]
struct ObjectParameters {
    model_matrix: [[f32; 4]; 4],
    prev_model_matrix: [[f32; 4]; 4],
    object_id: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, VertexType)]
struct MyVertexType {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    texcoord: [f32; 2],
}

#[derive(ShaderInterface)]
struct MeshShaderInterface {
    #[texture_binding(index = "0")]
    texture: gfx::SampledTexture2D,
    #[vertex_buffer(index = "0")]
    vertices: gfx::BufferSlice<[MyVertexType]>,
    #[index_buffer]
    indices: gfx::BufferSlice<[u32]>,
    #[render_target(index = "0")]
    diffuse: gfx::SampledTexture2D,
    #[uniform_buffer(index = "0")]
    camera_params: gfx::BufferSlice<CameraParameters>,
    #[uniform_buffer(index = "1")]
    object_params: gfx::BufferSlice<ObjectParameters>,
}

impl CameraParameters {
    pub fn from_camera(cam: &Camera, temporal_aa_offset: (f32, f32)) -> CameraParameters {
        let view_matrix = cam.view.to_homogeneous();
        let proj_matrix = cam.projection.unwrap();
        let viewproj_matrix = proj_matrix * view_matrix;
        let inverse_proj_matrix = cam.projection.inverse();

        CameraParameters {
            view_matrix: view_matrix.as_ref().clone(),
            proj_matrix: proj_matrix.as_ref().clone(),
            viewproj_matrix: viewproj_matrix.as_ref().clone(),
            inverse_proj_matrix: inverse_proj_matrix.as_ref().clone(),
            viewproj_matrix_velocity: viewproj_matrix.as_ref().clone(),
            prev_viewproj_matrix_velocity: viewproj_matrix.as_ref().clone(),
            temporal_aa_offset: [0.0; 2], // TODO
        }
    }
}

fn dump_shader_interface<T: gfx::ShaderInterface>() {
    let interface_desc = <T as gfx::ShaderInterface>::get_description();
    let uniform_constants = interface_desc.get_uniform_constants();
    let render_targets = interface_desc.get_render_targets();
    let vertex_buffers = interface_desc.get_vertex_buffers();
    let index_buffer = interface_desc.get_index_buffer();
    let texture_bindings = interface_desc.get_texture_bindings();

    debug!(
        "vertex layout = {:#?}",
        <MyVertexType as gfx::VertexType>::get_layout()
    );
    debug!("texture bindings: {:#?}", texture_bindings);
    debug!("uniform constants: {:#?}", uniform_constants);
    debug!("render targets: {:#?}", render_targets);
    debug!("vertex buffers: {:#?}", vertex_buffers);
    debug!("index buffer: {:#?}", index_buffer);
}

fn pick_one_file() -> Option<String> {
    let result = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
        panic!(e);
    });

    match result {
        nfd::Response::Okay(file_path) => Some(file_path),
        nfd::Response::OkayMultiple(files) => None,
        nfd::Response::Cancel => None,
    }
}

//
fn load_tex2d<P: AsRef<Path>>(
    ctx: &gfx::Context,
    path: P,
) -> Result<gfx::Texture2D, failure::Error> {
    let img = image::open(path)?;
    let (width, height) = img.dimensions();
    let format = match img.color() {
        image::ColorType::RGB(8) => gfx::Format::R8G8B8_SRGB,
        image::ColorType::RGBA(8) => gfx::Format::R8G8B8A8_SRGB,
        _ => return Err(format_err!("Unsupported ColorType")),
    };
    let bytes: &[u8] = match img {
        image::DynamicImage::ImageLuma8(_) => return Err(format_err!("Unsupported ColorType")),
        image::DynamicImage::ImageLumaA8(_) => return Err(format_err!("Unsupported ColorType")),
        image::DynamicImage::ImageRgb8(ref rgb) => &*rgb,
        image::DynamicImage::ImageRgba8(ref rgba) => &*rgba,
    };

    Ok(gfx::Texture2D::with_pixels(
        ctx,
        &gfx::Texture2DDesc::simple(format, width, height),
        &bytes,
    ))
}

struct Scene {
    ids: IdTable,
    objects: SceneObjects,
    root_obj: ID,
}

impl Scene {
    fn load<P: AsRef<Path>>(
        gctx: &gfx::Context,
        cache: &Cache,
        scene_file: P,
    ) -> Result<Scene, failure::Error> {
        let mut ids = IdTable::new();
        let mut objects = SceneObjects::new();
        let root_obj =
            scene_loader::load_scene_file(scene_file, &mut ids, gctx, cache, &mut objects)?;

        Ok(Scene {
            ids,
            objects,
            root_obj,
        })
    }
}

struct FrameInfo<'f> {
    frame: &'f gfx::Frame<'f>,
    framebuffer: &'f gfx::Framebuffer,
    dt: f64,
    frame_index: u64,
    aspect_ratio: f32,
}

struct State<'c> {
    tex_offset: [f32; 2],
    tex_rotation: f32,
    tex_scale: f32,
    tex_file: String,
    texture: Option<gfx::Texture2D>,
    context: gfx::Context,
    bgcolor: [f32; 4],
    cache: &'c Cache,
    scene_file: String,
    scene: Option<Scene>,
    camera_control: CameraControl,
    interpolation_mode: i32,
    mesh_shader: Option<gfx::TypedGraphicsPipeline<MeshShaderInterface>>,
}

impl<'c> State<'c> {
    fn new(context: &gfx::Context, cache: &'c Cache) -> State<'c> {
        dump_shader_interface::<MeshShaderInterface>();

        let mut state = State {
            tex_offset: [0.0f32; 2],
            tex_rotation: 0.0,
            tex_scale: 1.0,
            tex_file: "data/img/missing_512.png".into(),
            texture: None,
            context: context.clone(),
            bgcolor: [0f32; 4],
            cache,
            scene_file: "data/scenes/truc.obj".into(),
            scene: None,
            camera_control: CameraControl::default(),
            interpolation_mode: 0,
            mesh_shader: None,
        };
        state
            .reload_pipelines()
            .map_err(|err| error!("Reload pipelines failed: {}", err));
        state
    }

    fn reload_scene(&mut self) {
        self.scene = Scene::load(&self.context, &self.cache, &self.scene_file).ok();
    }

    fn reload_pipelines(&mut self) -> Result<(), Error> {
        let mesh_shader = GraphicsPipelineBuilder::new()
            .with_glsl_file_via_spirv("data/shaders/deferred.glsl")?
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
            .build(&self.context)?
            .into_typed::<MeshShaderInterface>()?;
        self.mesh_shader = Some(mesh_shader);
        Ok(())
    }

    fn reload_tex(&mut self) {
        self.texture = load_tex2d(&self.context, &self.tex_file).ok();
    }

    fn setup_camera(&mut self, frame_info: &FrameInfo) {
        if let Some(ref scene) = self.scene {
            // center camera on root object
            let root_bounds = scene
                .objects
                .get(scene.root_obj)
                .unwrap()
                .borrow()
                .world_bounds;
            self.camera_control
                .set_aspect_ratio(frame_info.aspect_ratio);
            let fovy = std::f32::consts::PI / 4.0f32;
            self.camera_control.center_on_aabb(root_bounds, fovy);
        }
    }

    fn render(&mut self, frame_info: &FrameInfo) {
        use autograph::gfx::DrawExt;

        let frame = frame_info.frame;
        let framebuffer = frame_info.framebuffer;
        frame.clear_framebuffer_color(framebuffer, 0, &self.bgcolor);
        // texture test
        let sampler = match self.interpolation_mode {
            0 => &gfx::NEAREST_CLAMP_SAMPLER,
            _ => &gfx::LINEAR_CLAMP_SAMPLER,
        };
        if let Some(ref tex) = self.texture {
            frame.blit_texture(
                framebuffer,
                tex,
                sampler,
                &RectTransform::new(
                    HorizontalAnchor::Left {
                        offset: self.tex_offset[0],
                        size: tex.width(),
                    },
                    VerticalAnchor::Top {
                        offset: self.tex_offset[1],
                        size: tex.height(),
                    },
                ).with_scale(self.tex_scale)
                    .with_rotation(self.tex_rotation),
            );
        }

        let camera = self.camera_control.camera();
    }

    fn render_ui<'ui>(&mut self, ui: &imgui::Ui<'ui>, frame_info: &FrameInfo) {
        // UI test
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), imgui::ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.small_button(im_str!("Load texture")) {
                    if let Some(s) = pick_one_file() {
                        self.tex_file = s;
                        self.reload_tex();
                    }
                }
                ui.same_line(0.0f32);
                ui.text_disabled(&imgui::ImString::new(self.tex_file.as_ref()));

                if ui.small_button(im_str!("Load scene")) {
                    if let Some(s) = pick_one_file() {
                        self.scene_file = s;
                        self.reload_scene();
                    }
                }
                ui.same_line(0.0f32);
                ui.text_disabled(&imgui::ImString::new(self.scene_file.as_ref()));

                ui.slider_float2(
                    im_str!("Texture offset"),
                    &mut self.tex_offset,
                    -100.0,
                    100.0,
                ).build();
                ui.slider_float(im_str!("Rotation"), &mut self.tex_rotation, -3.14, 3.14)
                    .build();
                ui.slider_float(im_str!("Scale"), &mut self.tex_scale, 0.1, 30.0)
                    .build();
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos.0,
                    mouse_pos.1
                ));
                ui.color_picker(im_str!("Background color"), &mut self.bgcolor)
                    .build();
                ui.combo(
                    im_str!("Interpolation mode"),
                    &mut self.interpolation_mode,
                    &[im_str!("Nearest"), im_str!("Linear")],
                    2,
                );
            });

        ui.main_menu_bar(|| {
            ui.menu(im_str!("Engine")).build(|| {
                ui.menu_item(im_str!("Take screenshot")).build();
            });
            ui.text(im_str!("Frame time: {}", frame_info.dt));
        });
    }
}

//==================================================================================================
//==================================================================================================
//==================================================================================================
fn main() {
    // Init logger
    pretty_env_logger::init().unwrap();
    // Init glutin, window with OpenGL context parameters
    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Autograph test")
        .with_dimensions((1280, 720).into());
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

    // takes ownership of event_loop and window
    let main_loop = MainLoop::new(
        &window,
        &gfx::ContextConfig {
            max_frames_in_flight: 3,
        },
    );

    // initialize the imgui state
    let (mut imgui, mut imgui_renderer, mut imgui_mouse_state) = imgui_glue::init(
        main_loop.context(),
        main_loop.cache(),
        Some("data/fonts/iosevka-regular.ttf"),
    );

    let mut state = State::new(main_loop.context(), main_loop.cache());
    let mut frame_index = 0u64;

    // start main loop
    main_loop.run(
        //=================================================================
        // RENDER
        |frame, framebuffer, dt| {
            let mut running = true;
            // poll events
            event_loop.poll_events(|event| {
                // forward to imgui
                imgui_glue::handle_event(&mut imgui, &event, &mut imgui_mouse_state);
                // should close
                match event {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => running = false,
                        _ => (),
                    },
                    _ => (),
                }
            });

            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::Enable(gl::FRAMEBUFFER_SRGB);
            }

            // get framebuffer dimensions and aspect ratio
            let (width, height) = framebuffer.size();
            let (fwidth, fheight) = (width as f32, height as f32);
            let aspect_ratio = fwidth / fheight;
            let frame_info = FrameInfo {
                frame,
                framebuffer,
                dt: dt.into(),
                frame_index,
                aspect_ratio,
            };

            state.render(&frame_info);

            frame_index += 1;

            // Create an IMGUI frame
            let ui = imgui.frame(
                window.get_inner_size().unwrap().into(),
                window.get_inner_size().unwrap().into(),
                dt,
            );

            state.render_ui(&ui, &frame_info);

            unsafe {
                gl::Disable(gl::FRAMEBUFFER_SRGB);
            }

            imgui_renderer.render(frame, framebuffer, ui);
            running
        },
    );
}
