use super::layout::{Bounds};
use super::{Arena, NodeId, RetainedNode};
use super::style::*;
use super::vdom::*;

use glutin::{GlWindow,EventsLoop};
use nanovg as nvg;
use yoga;

pub struct Renderer {
    context: nvg::Context,
    default_font_name: String,
    default_font_size: f32,
    last_clip_tree: Option<ClipTree>
}

// Clip tree example:
// Elements are placed in 2D+Z,
// Leaf nodes are elements, inner nodes are clips.
// Hit-test returns all elements under the pointer.

// where to put the clip tree:
// - inside the retained DOM: cannot reorder children by z-index on draw & hit-test (must collect & reorder separately)
// - outside: layout is either duplicated (between yoga, retained DOM, clip tree), or layout is inaccessible in event handlers (but it must be, for sliders)
//      - send layout separately when hit-testing?
//      - must have layout on key events also
//      - store layout in hash map (ID -> layout)
//      - prefer outside, as hit-testing must be efficient, and copy layout

/// A clip tree: a data structure for fast hit-testing of UI elements.
// TODO actually optimize, now it's the simplest tree structure.
// TODO avoid rebuilding the clip tree on each frame.
// TODO allow partial rebuilds of the clip tree.
struct ClipTree
{
    root: ClipTreeNode
}

/// A node of a clip tree.
#[derive(Clone,Debug)]
struct ClipTreeNode
{
    /// Associated node.
    id: NodeId,
    /// Object bounds (copied from the node).
    bounds: Bounds,
    /// Child elements, sorted by Z-order.
    children: Vec<ClipTreeNode>,
    /// Clip info (copied from the node). Contains the Z-order of the node and hit masks.
    clip_info: ClipInfo
}

impl ClipTreeNode
{
    /*fn get_bounds<'a>(&self, nodes: &'a Arena<RetainedNode>) -> &'a Bounds {
        &nodes[self.id].data().layout
    }*/
}

/// Finds all nodes of the clip tree that contain the point specified by pos.
/// Returns the results in hits.
fn hit_test_node(node: &ClipTreeNode, pos: (f32,f32), hits: &mut Vec<NodeId>)
{
    let hit = node.bounds.is_point_inside(pos);

    // there is a hit, and the node does not ignore hits => add it to the list.
    if hit || !node.clip_info.no_hits {
        hits.push(node.id);
    }

    // this node wasn't hit, but it doesn't clip its children:
    // they can still be hit themselves if they fall outside this node,
    // so hit-test them anyway.
    if !hit || !node.clip_info.clip {
        for c in node.children.iter() {
            hit_test_node(c, pos, hits);
        }
    }
}

impl ClipTree
{
    /// Returns the list of all nodes that contain the point specified in `pos`.
    pub fn hit_test(&self, pos: (f32,f32)) -> Vec<NodeId>
    {
        let mut hits = Vec::new();
        hit_test_node(&self.root, pos, &mut hits);
        hits
    }
}

/// Recursive function that builds a clip tree.
fn build_clip_tree_rec(
    nodes: &Arena<RetainedNode>,
    id: NodeId,
    parent_bounds: &Bounds) -> ClipTreeNode
{
    let mut clip_node = {
        let node = &nodes[id];
        let data = node.data();
        // get bounds
        let bounds = data.bounds;

        // let bounds = Bounds::from_yoga_layout(parent_bounds, data.flex.get_layout());
        // update bounds in the node itself
        // data.bounds = bounds;

        ClipTreeNode {
            // copy bounds in the clip tree for fast access
            clip_info: data.clip_info,
            id,
            bounds,
            children: Vec::new(),
        }
    };

    // recurse over children
    let mut next = node.first_child();
    let mut children = Vec::new();
    while let Some(id) = next {
        children.push(build_clip_tree_rec(nodes, id, &clip_node.bounds));
        next = nodes[id].next_sibling();
    }

    // sort children by Z-order.
    children.sort_by(|a,b| { a.clip_info.z.cmp(&b.clip_info.z) });
    clip_node.children = children;
    clip_node
}

/// Root function that builds the clip tree.
fn build_clip_tree(nodes: &Arena<RetainedNode>, root: NodeId, bounds: &Bounds) -> ClipTree
{
    ClipTree {
        root: build_clip_tree_rec(nodes, root, bounds)
    }
}

/// Default font path.
// FIXME: Get this from config.
// FIXME: Get this from font name.
const DEFAULT_FONT: &'static str = "data/fonts/iosevka-regular.ttf";

/// Renders a rectangle to a NanoVG frame.
fn render_rect(
    frame: &nvg::Frame,
    bounds: &Bounds,
    style: &Styles)
{
    let width = bounds.width();
    let height = bounds.height();

    // convert style to nvg styles.
    // background -> fill
    let fill_paint = {
        let (r, g, b, a) = style.non_layout.background_color;
        nvg::Color::new(r, g, b, a)
    };

    // border -> stroke.
    let stroke_paint = {
        let (r, g, b, a) = style.non_layout.border_color.top;
        nvg::Color::new(r, g, b, a)
    };

    let border_width = style.non_layout.border_width.top;
    let stroke_opts = nvg::StrokeOptions {
        width: border_width,
        ..Default::default()
    };
    let border_radius = style.non_layout.border_radius;

    //debug!("draw layout: {:?}", layout);

    // fill path (background)
    frame.path(
        |path| {
            if border_radius != 0.0 {
                path.rounded_rect(
                    (bounds.left, bounds.top),
                    (width, height),
                    border_radius,
                );
            } else {
                path.rect((bounds.left, bounds.top), (width, height));
            }
            path.fill(fill_paint, Default::default());
        },
        Default::default(),
    );

    // stroke path (border)
    frame.path(
        |path| {
            if border_radius != 0.0 {
                path.rounded_rect(
                    (bounds.left, bounds.top),
                    (width, height),
                    border_radius,
                );
            } else {
                path.rect(
                    (
                        bounds.left + 0.5 * border_width,
                        bounds.top + 0.5 * border_width,
                    ),
                    (
                        width - border_width,
                        height - border_width,
                    ),
                );
            }
            path.stroke(stroke_paint, stroke_opts);
        },
        Default::default(),
    );
}

/// Renders text.
fn render_text(
    frame: &nvg::Frame,
    font: nvg::Font,    // font reference
    text: &str,
    bounds: &Bounds,
    style: &Styles)
{
    frame.text(
        font,
        (bounds.left, bounds.top),
        text,
        // FIXME: handle text styles and different fonts
        nvg::TextOptions {
            color: nvg::Color::new(1.0, 1.0, 1.0, 1.0),
            size: 16.0,
            ..Default::default()
        },
    );
}

/// Renders a node from the DOM.
pub(super) fn render_node(
    frame: &nvg::Frame,
    font: nvg::Font,
    nodes: &mut Arena<RetainedNode>,
    clip_node: &ClipTreeNode)
{
    {
        let node = &mut nodes[clip_node.id];
        let data = node.data_mut();
        let styles = data.styles.as_ref().expect("styles were not computed before render");

        match data.contents {
            Contents::Text(ref text) => {
                render_text(frame, font, text, &clip_node.bounds, styles);
            },
            Contents::Element => {
                render_rect(frame, &clip_node.bounds, styles);
            }
        };
        // drop arena borrows
    }

    // We use the clip tree to iterate over children because we want to draw them in Z-order.
    for c in clip_node.children.iter() {
        render_node(frame, font, nodes, c);
    }
}


impl Renderer {
    /// Creates a new renderer.
    pub fn new(main_window: &GlWindow, events_loop: &EventsLoop) -> Renderer
    {
        // Create nanovg context
        let context = nvg::ContextBuilder::new()
            .stencil_strokes()
            .build()
            .expect("Initialization of NanoVG failed!");

        // load default font
        // TODO specify default font as config option
        // FIXME find default system font?
        let iosevka_font = nvg::Font::from_file(
            &context,
            "default",
            DEFAULT_FONT,
        ).expect("Failed to load default font");

        Renderer {
            context,
            default_font_name: "default".to_owned(),
            default_font_size: 14.0,
            last_clip_tree: None,
            //image_cache,
        }
    }

    /// Layouts the DOM and renders it to the specified window with OpenGL.
    pub fn layout_and_render_dom(&mut self, window: &GlWindow, nodes: &mut Arena<RetainedNode>, root: NodeId)
    {
        // get window bounds
        let bounds = {
            let root_node = &mut nodes[root];
            let hidpi_factor = window.get_hidpi_factor();
            let window_size: (u32, u32) = window.get_inner_size().unwrap().to_physical(hidpi_factor).into();
            let window_size = (window_size.0 as f32, window_size.0 as f32);
            // ask yoga to calculate the layout
            root_node.data_mut().flex.calculate_layout(window_size.0, window_size.1, yoga::Direction::LTR);
            Bounds {
                left: 0.0,
                right: window_size.0,
                bottom: window_size.1,
                top: 0.0,
            }
            // drop arena borrows
        };
        // build the clip tree
        let clip_tree = build_clip_tree(nodes, root, &bounds);
        // fetch the default font
        let font = nvg::Font::find(&self.context, self.default_font_name).unwrap();
        // recursively render elements
        self.context.frame((window_size.0 as f32, window_size.1 as f32), hidpi_factor as f32, |frame| {
            render_node(&frame, font, nodes, &clip_tree.root);
        });
        // set the clip tree for subsequent hit-test requests
        self.last_clip_tree = Some(clip_tree);
    }

    /// Performs hit-testing of the last DOM submitted with `layout_and_render_dom`.
    /// Returns the list of all nodes that contain the point specified in `pos`.
    pub fn hit_test(&self, pos: (f32,f32)) -> Vec<NodeId>
    {
        if let Some(ref clip_tree) = self.last_clip_tree {
            clip_tree.hit_test(pos)
        } else {
            warn!("Hit-test called with no clip tree");
            vec![]
        }
    }
}

impl Renderer {
    fn draw_image(&mut self, image_path: &str, layout: &Bounds) {
        /*let img = self.image_cache.get_or_load_image(image_path);
        img.map(|img| {
            self.frame.path(
                |path| {
                    let (w, h) = (layout.width(), layout.height());
                    path.rect((layout.left, layout.top), (layout.width(), layout.height()));
                    let pattern = nvg::ImagePattern {
                        image: img.as_ref(),
                        origin: (0.0, 0.0),
                        size: (w, h),
                        angle: 0.0,
                        alpha: 1.0,
                    };
                    path.fill(pattern, Default::default());
                },
                Default::default(),
            );
        });*/
    }
}

impl Renderer {
    /*fn measure_text(&self, text: &str, _style: &CachedStyle) -> f32 {
        let (advance, _bounds) = self.frame.text_bounds(
            self.default_font,
            (0.0, 0.0),
            text,
            nvg::TextOptions {
                size: self.default_font_size,
                ..Default::default()
            },
        );
        //debug!("text {} advance {}", text, advance);
        advance
    }

    fn measure_image(&self, img: &str) -> Option<(f32, f32)> {
        /*let img = self.image_cache.get_or_load_image(img);
        img.map(|img| {
            let (w, h) = img.size();
            (w as f32, h as f32)
        })*/
        unimplemented!()
    }*/
}
