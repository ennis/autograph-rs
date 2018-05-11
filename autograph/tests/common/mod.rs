extern crate autograph;
extern crate failure;
extern crate glutin;
extern crate image;
extern crate time;
extern crate winit;

use self::autograph::gfx;
use self::autograph::gfx::glsl::interface::{verify_spirv_interface,
                                            ShaderInterfaceVerificationError};
use self::autograph::gfx::glsl::{compile_glsl_to_spirv, preprocess_combined_shader_source,
                                 SourceWithFileName, SpirvModules};
use self::autograph::gfx::shader_interface::{ShaderInterface, ShaderInterfaceDesc};
use self::autograph::gfx::GraphicsShaderPipeline;
use self::autograph::gl;
use self::failure;
use self::glutin;
use self::glutin::GlContext;
use self::image;
use self::time;
use std::fs::File;
use std::io::Read;
use std::os::raw::c_void;
use std::path::Path;

pub struct TestWindowConfig {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
}

pub struct TestFrameInfo<'a, 'q: 'a> {
    pub frame: &'a mut gfx::Frame<'q>,
    pub framebuffer: &'a gfx::Framebuffer,
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: f32,
    pub frame_index: u64,
    pub delta_s: f32,
}

pub fn run_test<F>(config: &TestWindowConfig, mut body: F)
where
    F: FnMut(&mut TestFrameInfo) -> bool,
{
    // Init logger
    //pretty_env_logger::init().unwrap();
    // Init glutin, window with OpenGL context parameters
    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title(config.name)
        .with_dimensions(config.width, config.height);
    let context_builder = glutin::ContextBuilder::new()
        .with_gl_profile(glutin::GlProfile::Core)
        .with_gl_debug_flag(true)
        .with_vsync(true)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 6)));
    let window = glutin::GlWindow::new(window_builder, context_builder, &event_loop).unwrap();
    // load GL function pointers
    autograph::gl::load_with(|s| window.get_proc_address(s) as *const std::os::raw::c_void);
    // Make current the OpenGL context associated to the window
    unsafe { window.make_current() }.unwrap();
    // Load OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    // create an instance of gfx::Context
    let context = gfx::Context::new(&gfx::ContextConfig {
        max_frames_in_flight: 3,
    });
    // create a queue
    let mut queue = gfx::Queue::new(&context);
    // timers
    let mut loop_start_time = time::PreciseTime::now();
    let mut frame_duration = time::Duration::seconds(0);
    let mut running = true;
    let mut frame_index = 0u64;
    // application loop
    while running {
        // get default framebuffer from GL window
        let framebuffer = gfx::Framebuffer::from_gl_window(&context, &window);
        // create the frame
        let mut frame = gfx::Frame::new(&mut queue);
        // last frame time in seconds
        let delta_s = 1.0 / 1_000_000_000.0 * frame_duration.num_nanoseconds().unwrap() as f32;
        // get framebuffer dimensions and aspect ratio
        let (width, height) = framebuffer.size();
        let (fwidth, fheight) = (width as f32, height as f32);
        let aspect_ratio = fwidth / fheight;

        {
            let mut frame_info = TestFrameInfo {
                frame: &mut frame,
                framebuffer: &framebuffer,
                frame_index,
                aspect_ratio,
                width,
                height,
                delta_s,
            };
            // poll events
            event_loop.poll_events(|event| {
                // should close
                match event {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::Closed => running = false,
                        _ => (),
                    },
                    _ => (),
                }
            });
            // exit loop if the user requested an exit
            if !running {
                break;
            }
            // Put target FBO in SRGB mode (expect a linear output)
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::Enable(gl::FRAMEBUFFER_SRGB);
            }
            // run body
            running = body(&mut frame_info);
        }

        // submit frame
        frame.submit();
        // swap buffers
        window.swap_buffers().unwrap();
        // Calculate frame time
        let end_time = time::PreciseTime::now();
        frame_duration = loop_start_time.to(end_time);
        loop_start_time = end_time;
        frame_index += 1;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn load_tex2d<P: AsRef<Path>>(
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

////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn load_spv_modules(src: &str) -> SpirvModules {
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

////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn dump_error(error: &failure::Error) {
    let mut fail = error.cause();
    eprintln!("error: {}", fail);
    while let Some(cause) = fail.cause() {
        eprintln!("Caused by: {}", cause);
        fail = cause;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn load_pipeline_and_check_interface<I: ShaderInterface>(src: &str) {
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

macro_rules! make_interface_test_shader {
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
