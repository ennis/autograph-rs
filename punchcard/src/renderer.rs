use super::layout::Layout;
use super::style::{Styles};
use super::id_tree::{Arena, NodeId};
use super::vdom::{RetainedNode, Contents};

use std::rc::Rc;

use euclid;
use webrender;
use webrender::api::*;
use glutin;
use glutin::GlWindow;
use glutin::GlContext;
use winit;
use gleam::gl;
use euclid::vec2;
use yoga;
use yoga::prelude::*;


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

    fn new_frame_ready(&self, _: DocumentId, _scrolled: bool, _composite_needed: bool, _render_time_ns: Option<u64>) {
        self.wake_up();
    }
}


/// A window with a webrender context.
pub(super) struct WebrenderContext
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
    pub(super) fn new(window: &GlWindow, events_loop: &glutin::EventsLoop) -> WebrenderContext
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

/// Renders a DOM.
pub(super) fn render_node(builder: &mut DisplayListBuilder, txn: &mut Transaction, arena: &mut Arena<RetainedNode>, id: NodeId, parent_layout: &Layout)
{
    let layout = {
        let node = &mut arena[id];
        let data = node.data_mut();
        let layout = data.update_layout(parent_layout);

        match data.contents {
            Contents::Text(ref text) => {
                // TODO
            },
            Contents::Element => {
                render_rect(builder, txn, id, &layout, data.styles.as_ref().expect("styles were not computed before render"));
            }
        };
        // return layout, drop borrow of arena
        layout
    };

    let mut next = arena[id].first_child();
    while let Some(id) = next {
        render_node(builder, txn, arena, id, &layout);
        next = arena[id].next_sibling();
    }
}

const WR_DOM_NODE_MARKER: u16 = 3333;

///
/// Submits a styled rectangle to a webrender DisplayListBuilder.
///
fn render_rect(builder: &mut DisplayListBuilder, txn: &mut Transaction, id: NodeId, layout: &Layout, styles: &Styles) {

    // Convert colors
    let fill_color = {
        let (r, g, b, a) = styles.non_layout.background_color;
        ColorF::new(r, g, b, a)
    };

    let border_color = {
        let (r, g, b, a) = styles.non_layout.border_color.top;
        ColorF::new(r, g, b, a)
    };

    // Create bounds for item
    let bounds = LayoutRect::new(LayoutPoint::new(layout.left, layout.top),
                                 // WR doesn't like zero sizes?
                                 LayoutSize::new(
                                     layout.width().max(1.0),
                                     layout.height().max(1.0)));
    let mut info = LayoutPrimitiveInfo::new(bounds);
    // set tag for hit-testing, marker is here to say that it's a visual generated by us.
    info.tag = Some((id.as_u64(), WR_DOM_NODE_MARKER));

    // Create clip region
    let clip = ComplexClipRegion {
        rect: bounds,
        // TODO multiple border radii
        radii: BorderRadius::uniform(styles.non_layout.border_radius),
        mode: ClipMode::Clip,
    };
    let clip_id = builder.define_clip(bounds, vec![clip], None);

    //--------------------------------------------------------------
    // CLIP ACTIVE
    builder.push_clip_id(clip_id);

    //debug!("rect={:?}, fill={:?}", info, fill_color);

    //--------------------------
    // BACKGROUND
    builder.push_rect(&info, fill_color);

    //--------------------------
    // BOX SHADOW - INSET
    if let Some(ref box_shadow) = styles.non_layout.box_shadow {
        // draw box shadow?
        let rect = LayoutRect::zero();
        let simple_box_bounds = bounds;
        let offset = vec2(box_shadow.vertical_offset, box_shadow.horizontal_offset);
        let color = {
            let (r, g, b, a) = box_shadow.color;
            ColorF::new(r, g, b, a)
        };
        let simple_border_radius = 0.0;
        let info = LayoutPrimitiveInfo::with_clip_rect(rect, bounds);

        builder.push_box_shadow(
            &info,
            simple_box_bounds,
            offset,
            color,
            box_shadow.blur_radius,
            box_shadow.spread,
            BorderRadius::uniform(simple_border_radius),
            box_shadow.clip_mode,
        );
    }

    //--------------------------
    // BORDER
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

    builder.push_border(&info, border_widths, border_details);

    builder.pop_clip_id();
    // OUT OF CLIP
    //--------------------------------------------------------------

}

/// Render the DOM to a window.
pub(super) fn layout_and_render_dom(
    window: &glutin::GlWindow,
    ctx: &mut WebrenderContext,
    arena: &mut Arena<RetainedNode>,
    id: NodeId)
{
    let framebuffer_size = window.get_inner_size().unwrap();
    let framebuffer_size = DeviceUintSize::new(framebuffer_size.0, framebuffer_size.1);
    let device_pixel_ratio = window.hidpi_factor();
    let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
    let root_layout = Layout { top: 0.0, left: 0.0, right: layout_size.width, bottom: layout_size.height };
    //debug!("root layout: {:?}", root_layout);

    let mut builder = DisplayListBuilder::new(ctx.pipeline_id, layout_size);
    let mut txn = Transaction::new();

    let bounds = LayoutRect::new(LayoutPoint::zero(), builder.content_size());
    let info = LayoutPrimitiveInfo::new(bounds);

    // do layout
    {
        let root_node = &mut arena[id];
        root_node.data_mut().flex.calculate_layout(layout_size.width, layout_size.height, yoga::Direction::LTR);
    }

    builder.push_stacking_context(
        &info,
        None,
        TransformStyle::Flat,
        MixBlendMode::Normal,
        Vec::new(),
        GlyphRasterSpace::Screen,
    );

    render_node(&mut builder, &mut txn, arena, id, &root_layout);

    builder.pop_stacking_context();

    txn.set_display_list(
        ctx.epoch,
        None,
        layout_size,
        builder.finalize(),
        true,
    );
    txn.set_root_pipeline(ctx.pipeline_id);
    txn.generate_frame();
    ctx.api.send_transaction(ctx.document_id, txn);

    ctx.renderer.update();
    ctx.renderer.render(framebuffer_size).unwrap();
    ctx.renderer.flush_pipeline_info();

    //renderer.render(framebuffer_size).unwrap();
    //let _ = renderer.flush_pipeline_info();
}

/// Performs hit-testing on the context.
pub(super) fn hit_test(ctx: &WebrenderContext, pos: WorldPoint) -> Vec<NodeId>
{
    let hit_test_results = ctx.api.hit_test(
        ctx.document_id,
        Some(ctx.pipeline_id),
        pos,
        HitTestFlags::FIND_ALL);

    hit_test_results.items.iter().filter_map(|item| {
        if item.tag.1 == WR_DOM_NODE_MARKER {
            Some(NodeId::from_u64(item.tag.0))
        } else {
            None
        }
    }).collect::<Vec<_>>()
}

//#[derive(Copy,Clone,Debug, Ord, PartialOrd, PartialEq, Eq)]
//pub struct WindowID(pub usize);
