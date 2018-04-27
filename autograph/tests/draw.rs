#[macro_use]
extern crate autograph;
#[macro_use]
extern crate autograph_derive;
#[macro_use]
extern crate failure;
extern crate glutin;
extern crate winit;
extern crate time;

use autograph::gfx;
use autograph::gl;
use glutin::GlContext;

struct TestWindowConfig
{
    name: &'static str,
    width: u32,
    height: u32
}

struct TestFrameInfo<'a, 'q:'a>
{
    frame: &'a mut gfx::Frame<'q>,
    framebuffer: &'a gfx::Framebuffer,
    width: u32,
    height: u32,
    aspect_ratio: f32,
    frame_index: u64,
    delta_s: f32
}

fn run_test<F>(config: &TestWindowConfig, mut body: F) where
    F: FnMut(&mut TestFrameInfo) -> bool
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
    autograph::gl::load_with(|s| { window.get_proc_address(s) as *const std::os::raw::c_void });
    // Make current the OpenGL context associated to the window
    unsafe { window.make_current() }.unwrap();
    // Load OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    // create an instance of gfx::Context
    let context = gfx::Context::new(&gfx::ContextConfig { max_frames_in_flight: 3 });
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
        let delta_s = 1.0/1_000_000_000.0 * frame_duration.num_nanoseconds().unwrap() as f32;
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
                delta_s
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
#[test]
fn test_simple_window()
{
    let cfg = TestWindowConfig {
        name: "simple_draw",
        width: 256,
        height: 256
    };

    run_test(&cfg, |frame_info| {
        false
    });
}
