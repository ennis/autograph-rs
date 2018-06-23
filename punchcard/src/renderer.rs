use super::layout::Layout;
use super::style::{Styles};
use super::vdom::{RetainedElement, RetainedData, Contents};

use euclid;
use webrender;
use webrender::api::*;
use glutin;
use glutin::GlWindow;
use glutin::GlContext;
use winit;
use gleam::gl;
use euclid::vec2;

use std::rc::Rc;

/// Holds a proxy to wake up the event loop when a frame is ready to render.
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


/// A window with a webrender context.
/// Owns the underlying window.
struct WebrenderContext
{
    renderer: webrender::Renderer,
    name: &'static str,
    pipeline_id: PipelineId,
    document_id: DocumentId,
    epoch: Epoch,
    api: RenderApi,
    //font_instance_key: FontInstanceKey,
}

impl WebrenderContext
{
    fn new(window: &GlWindow, events_loop: &glutin::EventsLoop) -> WebrenderContext
    {
        let device_pixel_ratio = window.hidpi_factor();
        let opts = webrender::RendererOptions {
            resource_override_path: None,
            precache_shaders: false,    // crashes if set?
            device_pixel_ratio,
            clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
            //scatter_gpu_cache_updates: false,
            debug_flags: webrender::DebugFlags::ECHO_DRIVER_MESSAGES,
            ..webrender::RendererOptions::default()
        };
        Self::new_with_options(window, events_loop, opts)
    }

    fn new_with_options(window: &GlWindow, events_loop: &glutin::EventsLoop, opts: webrender::RendererOptions) -> WebrenderContext
    {
        //========================================================================
        // GL API
        let gl = match window.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => {
                panic!("unsupported OpenGL API")
            }
        };

        debug!("OpenGL version {}", gl.get_string(gl::VERSION));
        //debug!("Shader resource path: {:?}", res_path);
        let device_pixel_ratio = window.hidpi_factor();
        debug!("HiDPI factor: {}", device_pixel_ratio);

        //========================================================================
        // webrender setup
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

        WebrenderContext {
            renderer,
            name: "webrender context",
            pipeline_id: PipelineId(0, 0),
            document_id,
            epoch: Epoch(0),
            api,
        }
    }
}

/// Custom wrapper around a webrender display list builder.
struct RenderFrame {
    builder: DisplayListBuilder,
    txn: Transaction
}

impl RenderFrame
{
    /// Calculate the layout of a DOM element.
    pub(super) fn layout_dom(&mut self, dom: &mut RetainedElement, parent_layout: &Layout) -> Layout
    {
        let layout = Layout::from_yoga_layout(parent_layout, dom.extra.flex.get_layout());
        dom.extra.layout = layout;
        debug!("calc layout: {:?}", layout);
        layout
    }

    /// Renders a DOM.
    pub(super) fn render_dom(&mut self, dom: &mut RetainedElement, parent_layout: &Layout)
    {
        // update layout
        let layout = self.layout_dom(dom, parent_layout);

        match dom.contents
        {
            Contents::Text(ref text) => {
                // TODO
            },
            Contents::Div(ref mut elements) => {
                self.draw_rect(&dom.extra.layout, dom.extra.styles.as_ref().expect("styles were not computed before render"));
                for elt in elements.iter_mut() {
                    self.render_dom(elt, &layout);
                }
            }
        }
    }

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
    }
}

/// Rendering is done by webrender.
/// Renderer holds all context needed by webrender as well as "side windows"
/// not managed by the user.
pub struct Renderer
{
    /// All contexts: each context represents a window.
    /// Context 0 is for main window. Context 1.. are for owned windows.
    main_context: WebrenderContext,
    /// Owned windows (created by the UI).
    side_windows: Vec<(GlWindow, WebrenderContext)>
}


#[derive(Copy,Clone,Debug, Ord, PartialOrd, PartialEq, Eq)]
pub struct WindowID(pub usize);

impl Renderer
{
    /// Creates a new renderer to the given window.
    pub fn new(main_window: &glutin::GlWindow, events_loop: &glutin::EventsLoop) -> Renderer {
        Renderer {
            main_context: WebrenderContext::new(main_window, events_loop),
            side_windows: Vec::new()
        }
    }

    /// Creates a new side-window, owned by the renderer.
    pub fn open_side_window(&mut self) -> WindowID {
        unimplemented!()
    }

    /// Closes a side window.
    pub fn close_side_window(&mut self, window: WindowID) {
        unimplemented!()
    }

    /// Gets a RenderFrame (a wrapper around a displaylistBuilder) for the specified window.
    /// The window is identified by an ID.
    pub(super) fn render_to_window(
        &mut self, window: WindowID,
        framebuffer_size: (u32,u32),
        device_pixel_ratio: f32,
        dom: &mut RetainedElement)
    {
        if window != WindowID(0) {
            unimplemented!()
        }

        let mut ctx = &mut self.main_context;

        let framebuffer_size = DeviceUintSize::new(framebuffer_size.0, framebuffer_size.1);
        let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);

        let mut builder = DisplayListBuilder::new(ctx.pipeline_id, layout_size);
        let mut txn = Transaction::new();

        let mut frame = RenderFrame {
            builder,
            txn
        };

        let bounds = LayoutRect::new(LayoutPoint::zero(), frame.builder.content_size());
        let info = LayoutPrimitiveInfo::new(bounds);
        frame.builder.push_stacking_context(
            &info,
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            Vec::new(),
            GlyphRasterSpace::Screen,
        );

        let root_layout = Layout { top: 0.0, left: 0.0, right: layout_size.width, bottom: layout_size.height };
        debug!("root layout: {:?}", root_layout);
        frame.render_dom(dom, &root_layout);

        frame.builder.pop_stacking_context();

        frame.txn.set_display_list(
            ctx.epoch,
            None,
            layout_size,
            frame.builder.finalize(),
            true,
        );
        frame.txn.set_root_pipeline(ctx.pipeline_id);
        frame.txn.generate_frame();
        ctx.api.send_transaction(ctx.document_id, frame.txn);

        ctx.renderer.update();
        ctx.renderer.render(framebuffer_size).unwrap();
        ctx.renderer.flush_pipeline_info();

        //renderer.render(framebuffer_size).unwrap();
        //let _ = renderer.flush_pipeline_info();
    }

    pub(super) fn measure_text(&self, text: &str, styles: &Styles) -> f32 {
        // TODO measure text in webrender?
        0.0
    }

    pub(super) fn measure_image(&self, image_path: &str) -> Option<(f32, f32)> {
        None
    }
}


// restrictions:
// - cannot own the window
// - does not create the window
//      issue: auxiliary windows? => ask client to create a new window+glcontext with a callback.
// - user swaps buffers manually
// needs:
// - an opengl context
