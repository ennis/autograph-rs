use autograph::gfx;
use autograph::cache::Cache;
use autograph::gl;
use autograph::gl::types::*;
use glutin;
use glutin::GlContext;
use time;
use std::sync::Arc;
use std::cell::RefCell;

// Scaffolding for the application
pub struct MainLoop<'a> {
    window: &'a glutin::GlWindow,
    pub cache: Cache,
    pub context: gfx::Context,
    pub queue: RefCell<gfx::Queue>,
}

const PRINT_FPS_EVERY_SECONDS: i64 = 3;

impl<'a> MainLoop<'a> {
    pub fn window(&self) -> &glutin::GlWindow {
        &self.window
    }

    pub fn context(&self) -> &gfx::Context {
        &self.context
    }

    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /*pub fn queue(&self) -> &gfx::Queue {
        &self.queue.borrow()
    }*/

    // takes ownership of the window and the event loop
    pub fn new<'b>(window: &'b glutin::GlWindow, config: &gfx::ContextConfig) -> MainLoop<'b> {
        // Make current the OpenGL context associated to the window
        unsafe { window.make_current() }.unwrap();

        // Load OpenGL function pointers
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        // create an instance of gfx::Context
        // NOTE: useless for now
        let context = gfx::Context::new(config);

        // create a queue
        let queue = gfx::Queue::new(&context);

        MainLoop {
            window,
            cache: Cache::new(),
            context,
            queue: RefCell::new(queue),
        }
    }

    pub fn run<F>(&self, mut body: F)
    where
        F: FnMut(&gfx::Frame, &gfx::Framebuffer, f32) -> bool,
    {
        let mut loop_start_time = time::PreciseTime::now();
        let mut last_debug_time = time::PreciseTime::now();
        let mut num_frames_since_last_print = 0;
        let mut frame_duration = time::Duration::seconds(0);
        let mut running = true;

        // imgui stuff
        while running {
            // get default framebuffer from GL window
            let default_framebuffer = gfx::Framebuffer::from_gl_window(
                &self.context,
                &self.window,
            );
            // create the frame
            let mut queue = self.queue.borrow_mut();
            let mut frame = gfx::Frame::new(&mut queue);
            // last frame time in seconds
            let delta_s = 1_000_000_000f32 * frame_duration.num_nanoseconds().unwrap() as f32;
            //let mut ui = imgui_glue.new_frame(window.get_inner_size_points().unwrap(), window.get_inner_size_pixels().unwrap(), delta_s);
            running = body(&mut frame, &default_framebuffer, delta_s);
            // render GUI (to default framebuffer)
            // imgui_renderer.render(&frame, default_framebuffer.clone(), &upload_buf, ui);
            // submit frame
            frame.submit();
            // swap buffers
            self.window.swap_buffers().unwrap();
            // Process all filesystem events that concern files used to load objects in the cache
            // May trigger reloads of some cached objects
            self.cache.process_filesystem_events();

            // Calculate frame time and average times and FPS over a certain time period
            // also print some stats to stdout
            num_frames_since_last_print += 1;
            let end_time = time::PreciseTime::now();
            frame_duration = loop_start_time.to(end_time);
            loop_start_time = end_time;
            let duration_since_last_print = last_debug_time.to(end_time);
            if last_debug_time.to(end_time) > time::Duration::seconds(PRINT_FPS_EVERY_SECONDS) {
                info!(
                    "Last frame time was {:?} ms ({:?} FPS) | average over {} frames: {:?} ms ({:?} FPS)",
                    frame_duration.num_milliseconds(),
                    1_000_000_000f32 / frame_duration.num_nanoseconds().unwrap() as f32,
                    num_frames_since_last_print,
                    duration_since_last_print.num_milliseconds() as f32 /
                        num_frames_since_last_print as f32,
                    num_frames_since_last_print as f32 * 1_000_000_000f32 /
                        duration_since_last_print.num_nanoseconds().unwrap() as f32,
                );
                num_frames_since_last_print = 0;
                last_debug_time = end_time;
            }
        }
    }
}
