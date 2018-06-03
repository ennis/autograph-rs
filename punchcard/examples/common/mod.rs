use glutin::GlContext;

use gl;
use glutin;
use nvg;
use pretty_env_logger;
use punchcard::*;
use std::path::{Path, PathBuf};
use std::env;

const INIT_WINDOW_SIZE: (u32, u32) = (1024, 720);

pub fn gui_test<F>(mut f: F)
where
    F: FnMut(&mut Ui),
{
    env::set_current_dir(env!("CARGO_MANIFEST_DIR"));
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

    let iosevka_font = nvg::Font::from_file(
        &context,
        "Iosevka",
        "data/fonts/iosevka-regular.ttf",
    ).expect("Failed to load font");

    let mut running = true;
    let mut ui = Ui::new();
    ui.load_stylesheet("data/css/default.css").expect("failed to load default stylesheet");
    debug!("HiDPI factor is {}", gl_window.hidpi_factor());

    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                ui.dispatch_event(&event);
                match event {
                    glutin::WindowEvent::CloseRequested => running = false,
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

        context.frame(
            (width as f32, height as f32),
            gl_window.hidpi_factor(),
            |frame| {
                f(&mut ui);
                let mut renderer = NvgRenderer::new(frame, iosevka_font, 16.0);
                ui.render((width as f32, height as f32), &mut renderer);
            },
        );

        gl_window.swap_buffers().unwrap();
    }
}
