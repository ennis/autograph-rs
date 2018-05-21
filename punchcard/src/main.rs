#[macro_use]
extern crate log;
#[macro_use]
extern crate pretty_env_logger;
extern crate glutin;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate cassowary;
extern crate diff;
extern crate gl;
extern crate indexmap;
extern crate nanovg as nvg;
extern crate num;
extern crate petgraph;
extern crate rand;
extern crate time;
#[macro_use]
extern crate yoga;
extern crate cssparser;
extern crate warmy;

use glutin::GlContext;
use std::f32::consts::PI;
use std::time::Instant;
use warmy::{Store,StoreOpt};

mod test_ui;
mod ui;

const INIT_WINDOW_SIZE: (u32, u32) = (1024, 720);

/*struct ImageCache<'ctx>
{
    context: &'ctx nvg::Context
}

struct Renderer<'cache, 'ctx:'cache>
{
    cache: &'cache ImageCache<'ctx>,
    font: nvg::Font<'ctx>,
    frame: nvg::Frame<'ctx>
}

impl<'cache, 'ctx: 'cache> Renderer<'cache, 'ctx>
{
    pub fn new(frame: nvg::Frame<'ctx>, font: nvg::Font<'ctx>, cache: &'cache ImageCache<'ctx>) -> Renderer<'cache, 'ctx> {
        Renderer {
            frame,
            font,
            cache
        }
    }
}*/

fn main() {
    pretty_env_logger::init();
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Glutin NanoVG")
        .with_dimensions(INIT_WINDOW_SIZE.0, INIT_WINDOW_SIZE.1);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4)
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let context = nvg::ContextBuilder::new()
        .stencil_strokes()
        .build()
        .expect("Initialization of NanoVG failed!");
    let iosevka_font = nvg::Font::from_file(&context, "Iosevka", "data/fonts/iosevka-regular.ttf")
        .expect("Failed to load font");

    let start_time = Instant::now();
    let mut running = true;
    let mut ui = ui::Ui::new();
    let mut data = 10i32;
    //let image_cache = ImageCache { context: &context };
    let image_cache = ui::ImageCache::new(&context);
    ui.load_stylesheet("data/css/default.css");
    debug!("HiDPI factor is {}", gl_window.hidpi_factor());

    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                ui.dispatch_event(&event);
                match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    _ => {}
                }
            }
            _ => {}
        });

        if !running {
            break;
        }

        let (width, height) = gl_window.get_inner_size().unwrap();

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }

        let elapsed = {
            let elapsed = start_time.elapsed();
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        } as f32;

        // Let's draw a frame!
        context.frame((width as f32, height as f32), gl_window.hidpi_factor(), |frame| {
            test_ui::make_ui(&mut ui, &mut data);
            //let mut renderer = Renderer::new(frame, iosevka_font, &image_cache);
            let mut renderer = ui::NvgRenderer::new(frame, iosevka_font, 16.0, &image_cache);
            ui.render((width as f32, height as f32), &mut renderer);
        });

        gl_window.swap_buffers().unwrap();
    }
}
