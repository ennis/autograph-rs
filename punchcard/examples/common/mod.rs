extern crate warmy;
extern crate glutin;
extern crate winit;
extern crate webrender;
extern crate euclid;
extern crate gleam;

use self::euclid::vec2;
use self::gleam::gl;
use self::glutin::GlContext;
use pretty_env_logger;
use punchcard::*;
use std::path::{Path, PathBuf};
use std::env;
use std::cmp::{min,max};
use self::webrender::api::*;

const INIT_WINDOW_SIZE: (u32, u32) = (1024, 720);

struct Notifier {
    events_proxy: winit::EventsLoopProxy,
}

impl Notifier {
    fn new(events_proxy: winit::EventsLoopProxy) -> Notifier {
        Notifier { events_proxy }
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<RenderNotifier> {
        Box::new(Notifier {
            events_proxy: self.events_proxy.clone(),
        })
    }

    fn wake_up(&self) {
        #[cfg(not(target_os = "android"))]
        let _ = self.events_proxy.wakeup();
    }

    fn new_frame_ready(&self, _: DocumentId, _scrolled: bool, _composite_needed: bool) {
        self.wake_up();
    }
}

/*fn ui_render(
    ui: &mut Ui,
    api: &RenderApi,
    builder: &mut DisplayListBuilder,
    txn: &mut Transaction,
    framebuffer_size: DeviceUintSize,
    pipeline_id: PipelineId,
    document_id: DocumentId)
{
    let mut wr_renderer = WRRenderer {
        api,
        builder,
        txn,
        framebuffer_size,
        pipeline_id,
        document_id,
    };

    debug!("fb size= {:?}", framebuffer_size);
    ui.render((framebuffer_size.width as f32, framebuffer_size.height as f32), &mut wr_renderer);
}*/

/*fn ui_event(
    ui: &mut Ui,
    event: winit::WindowEvent,
    api: &RenderApi,
    document_id: DocumentId) -> bool
{
    ui.dispatch_event(&event);
    // don't redraw
    false
}*/

struct WRRenderer<'a>
{
    api: &'a RenderApi,
    builder: &'a mut DisplayListBuilder,
    txn: &'a mut Transaction,
    framebuffer_size: DeviceUintSize,
    pipeline_id: PipelineId,
    document_id: DocumentId
}

impl<'a> WRRenderer<'a>
{
    fn draw_rect(&mut self, layout: &Layout, styles: &Styles) {
        let fill_color = {
            let (r, g, b, a) = styles.non_layout.background_color;
            ColorF::new(r, g, b, a)
        };

        let border_color = {
            let (r, g, b, a) = styles.non_layout.border_color.top;
            ColorF::new(r, g, b, a)
        };

        let bounds = LayoutRect::new(LayoutPoint::new(layout.left, layout.top),
                                     // WR doesn't like zero sizes?
                                     LayoutSize::new(
                                        layout.width().max(1.0),
                                        layout.height().max(1.0)));
        let info = LayoutPrimitiveInfo::new(bounds);

        let clip = ComplexClipRegion {
            rect: bounds,
            radii: BorderRadius::uniform(styles.non_layout.border_radius),
            mode: ClipMode::Clip,
        };
        let clip_id = self.builder.define_clip(bounds, vec![clip], None);
        self.builder.push_clip_id(clip_id);

        self.builder.push_rect(&info, fill_color);

        let border_side = BorderSide {
            color: border_color,
            style: BorderStyle::Solid,
        };
        let border_widths = BorderWidths {
            top: styles.non_layout.border_width.top.max(1.0),
            left: styles.non_layout.border_width.left.max(1.0),
            bottom: styles.non_layout.border_width.bottom.max(1.0),
            right: styles.non_layout.border_width.right.max(1.0)
        };
        let border_details = BorderDetails::Normal(NormalBorder {
            top: border_side,
            right: border_side,
            bottom: border_side,
            left: border_side,
            radius: BorderRadius::uniform(styles.non_layout.border_radius),
        });

        self.builder.push_border(&info, border_widths, border_details);
        self.builder.pop_clip_id();

        // draw box shadow?
        /*let rect = LayoutRect::zero();
        let offset = vec2(10.0, 10.0);
        let color = ColorF::new(1.0, 1.0, 1.0, 1.0);
        let blur_radius = 0.0;
        let spread_radius = 0.0;
        let simple_border_radius = 8.0;
        let box_shadow_type = BoxShadowClipMode::Inset;
        let info = LayoutPrimitiveInfo::with_clip_rect(rect, bounds);

        self.builder.push_box_shadow(
            &info,
            bounds,
            offset,
            color,
            blur_radius,
            spread_radius,
            BorderRadius::uniform(simple_border_radius),
            box_shadow_type,
        );*/
    }
}

impl<'a> Renderer for WRRenderer<'a>
{
    fn measure_text(&self, text: &str, styles: &Styles) -> f32 {
        // TODO measure text in webrender?
        0.0
    }

    fn measure_image(&self, image_path: &str) -> Option<(f32, f32)> {
        None
    }

    fn draw_frame(&mut self, items: &[DrawItem]) {
        let bounds = LayoutRect::new(LayoutPoint::zero(), self.builder.content_size());
        let info = LayoutPrimitiveInfo::new(bounds);
        self.builder.push_stacking_context(
            &info,
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            Vec::new(),
            GlyphRasterSpace::Screen,
        );

        for di in items {
            match di.kind {
                DrawItemKind::Rect => {
                    self.draw_rect(&di.layout, &di.styles);
                }
                DrawItemKind::Image(_) => unimplemented!(),
                DrawItemKind::Text(ref str) => {
                    //self.draw_text(str, &di.layout, &di.style);
                }
            }
        }

        self.builder.pop_stacking_context();
    }
}


pub fn main_wrapper(title: &str, width: u32, height: u32, mut f: impl FnMut(&mut DomSink))
{
    env::set_current_dir(env!("CARGO_MANIFEST_DIR"));
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();
    let res_path = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    //========================================================================
    //========================================================================
    // Window & GL context setup
    // ========================================================================
    // ========================================================================
    let mut events_loop = winit::EventsLoop::new();
    let context_builder = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl(glutin::GlRequest::GlThenGles {
            opengl_version: (4, 6),
            opengles_version: (3, 0),
        });
    let window_builder = winit::WindowBuilder::new()
        .with_title(title)
        .with_multitouch()
        .with_dimensions(width, height);
    let window = glutin::GlWindow::new(window_builder, context_builder, &events_loop)
        .unwrap();

    unsafe {
        window.make_current().ok();
    }

    let gl = match window.get_api() {
        glutin::Api::OpenGl => unsafe {
            gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
        },
        glutin::Api::OpenGlEs => unsafe {
            gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
        },
        glutin::Api::WebGl => unimplemented!(),
    };


    //========================================================================
    //========================================================================
    // Webrender setup
    //========================================================================
    //========================================================================
    println!("OpenGL version {}", gl.get_string(gl::VERSION));
    println!("Shader resource path: {:?}", res_path);
    let device_pixel_ratio = window.hidpi_factor();
    println!("HiDPI factor: {}", device_pixel_ratio);

    println!("Loading shaders...");
    let opts = webrender::RendererOptions {
        resource_override_path: res_path,
        precache_shaders: false,
        device_pixel_ratio,
        clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
        //scatter_gpu_cache_updates: false,
        debug_flags: webrender::DebugFlags::ECHO_DRIVER_MESSAGES,
        ..webrender::RendererOptions::default()
    };

    let framebuffer_size = {
        let (width, height) = window.get_inner_size().unwrap();
        DeviceUintSize::new(width, height)
    };
    let notifier = Box::new(Notifier::new(events_loop.create_proxy()));
    let (mut renderer, sender) = webrender::Renderer::new(gl.clone(), notifier, opts).unwrap();
    let api = sender.create_api();
    let document_id = api.add_document(framebuffer_size, 0);

    let (external, output) = (None,None); //example.get_image_handlers(&*gl);

    if let Some(output_image_handler) = output {
        renderer.set_output_image_handler(output_image_handler);
    }

    if let Some(external_image_handler) = external {
        renderer.set_external_image_handler(external_image_handler);
    }

    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
    let mut txn = Transaction::new();

    //========================================================================
    //========================================================================
    // UI
    //========================================================================
    //========================================================================
    let mut ui = Ui::new();
    ui.load_stylesheet("data/css/default.css");

    //========================================================================
    //========================================================================
    // Event loop
    //========================================================================
    //========================================================================

    // initial render
    ui.update(|dom| f(dom));


    /*ui_render(
        &mut ui,
        &api,
        &mut builder,
        &mut txn,
        framebuffer_size,
        pipeline_id,
        document_id,
    );*/
    txn.set_display_list(
        epoch,
        None,
        layout_size,
        builder.finalize(),
        true,
    );
    txn.set_root_pipeline(pipeline_id);
    txn.generate_frame();
    api.send_transaction(document_id, txn);

    println!("Entering event loop");

    loop {
        let frame_time = measure_time(|| {
            let mut txn = Transaction::new();

            events_loop.poll_events(|global_event| {
                match global_event {
                    winit::Event::WindowEvent {
                        event: winit::WindowEvent::CloseRequested,
                        ..
                    } => {},
                    winit::Event::WindowEvent {
                        event: winit::WindowEvent::KeyboardInput {
                            input: winit::KeyboardInput {
                                state: winit::ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            },
                            ..
                        },
                        ..
                    } => match key {
                        winit::VirtualKeyCode::Escape => {},
                        winit::VirtualKeyCode::P => renderer.toggle_debug_flags(webrender::DebugFlags::PROFILER_DBG),
                        winit::VirtualKeyCode::O => renderer.toggle_debug_flags(webrender::DebugFlags::RENDER_TARGET_DBG),
                        winit::VirtualKeyCode::I => renderer.toggle_debug_flags(webrender::DebugFlags::TEXTURE_CACHE_DBG),
                        winit::VirtualKeyCode::S => renderer.toggle_debug_flags(webrender::DebugFlags::COMPACT_PROFILER),
                        winit::VirtualKeyCode::Q => renderer.toggle_debug_flags(
                            webrender::DebugFlags::GPU_TIME_QUERIES | webrender::DebugFlags::GPU_SAMPLE_QUERIES
                        ),
                        winit::VirtualKeyCode::Key1 => txn.set_window_parameters(
                            framebuffer_size,
                            DeviceUintRect::new(DeviceUintPoint::zero(), framebuffer_size),
                            1.0
                        ),
                        winit::VirtualKeyCode::Key2 => txn.set_window_parameters(
                            framebuffer_size,
                            DeviceUintRect::new(DeviceUintPoint::zero(), framebuffer_size),
                            2.0
                        ),
                        winit::VirtualKeyCode::M => api.notify_memory_pressure(),
                        #[cfg(feature = "capture")]
                        winit::VirtualKeyCode::C => {
                            let path: PathBuf = "../captures/example".into();
                            //TODO: switch between SCENE/FRAME capture types
                            // based on "shift" modifier, when `glutin` is updated.
                            let bits = CaptureBits::all();
                            api.save_capture(path, bits);
                        },
                        _ => {
                            let win_event = match global_event {
                                winit::Event::WindowEvent { event, .. } => event,
                                _ => unreachable!()
                            };
                            //ui_event(&mut ui, win_event, &api, document_id);
                        },
                    },
                    winit::Event::WindowEvent { event, .. } => {
                        //ui_event(&mut ui, event, &api, document_id);
                    },
                    _ => {},
                };
            });

            ui.update(|dom| f(dom));

            let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);

            /*ui_render(
                &mut ui,
                &api,
                &mut builder,
                &mut txn,
                framebuffer_size,
                pipeline_id,
                document_id,
            );*/
            txn.set_display_list(
                epoch,
                None,
                layout_size,
                builder.finalize(),
                true,
            );
            txn.generate_frame();
            api.send_transaction(document_id, txn);

            renderer.update();
            renderer.render(framebuffer_size).unwrap();
            let _ = renderer.flush_pipeline_info();

            //example.draw_custom(&*gl);

            window.swap_buffers().ok();
        });
        debug!("frame time: {}us", frame_time);
        // target 60fps
        /*if frame_time < 1_000_000 / 60 {
            ::std::thread::sleep(::std::time::Duration::from_micros(1_000_000 / 60 - frame_time));
        }*/
    }

    renderer.deinit();
}

// Supporting multi-window and other stuff
// - Should be transparent to the user
// - let ownership of the event loop to the user
// - must provide an interface to create a window
// - *** or just use winit+webrender internally
// - pass ref to event_loop to Ui::new(&event_loop, context_parameters)
// - Ui::set_context_parameters() for the GL (or Vulkan?) context of the created platform windows.
// - Then, call Ui::platform_window(|ui| {
//      window.width = XXX;
//      window.height = XXX;
//      window.title = XXX;
//      window.show_decorations = XXX;
// })
// - depends on the simplification / refactor of UI specification
