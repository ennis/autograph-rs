#[macro_use]
extern crate log;
extern crate glutin;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate indexmap;
extern crate nanovg as nvg;
extern crate num;
extern crate petgraph;
extern crate rand;
extern crate time;
extern crate yoga;
extern crate cssparser;
extern crate warmy;
extern crate winapi;

use failure::Error;
use std::path::{Path};
use indexmap::IndexMap;

mod behavior;
mod container;
mod css;
mod id_stack;
mod input;
mod item;
mod layout;
mod nanovg_renderer;
mod renderer;
mod sizer;
mod style;
mod widgets;

pub use self::behavior::{Behavior, CheckboxBehavior, DragBehavior, DragState};
pub use self::container::UiContainer;
pub use self::css::Stylesheet;
pub use self::id_stack::{IdStack, ItemID};
pub use self::input::InputState;
pub use self::item::Item;
pub use self::layout::{ContentMeasurement, Layout};
pub use self::nanovg_renderer::NvgRenderer;
pub use self::renderer::{DrawItem, DrawItemKind, DrawList, Renderer};
pub use self::style::{
    Background, CachedStyle, Color, ComputedStyle, LinearGradient, RadialGradient,
};
pub use self::widgets::*;
pub use glutin::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
pub use yoga::prelude::*;

use self::input::{DispatchChain, DispatchTarget, PointerCapture};
use self::item::{ItemChildren, ItemNode};
use self::style::apply_to_flex_node;
pub use warmy::{FSKey, Res, Store, StoreOpt};

/*macro_rules! unwrap_enum {
    ($e:expr,ref mut $p:path) => {
        match $e {
            $p(ref mut e) => e,
            _ => panic!("unexpected enum variant"),
        }
    };
    ($e:expr,ref $p:path) => {
        match $e {
            $p(ref e) => e,
            _ => panic!("unexpected enum variant"),
        }
    };
    ($e:expr, $p:path) => {
        match $e {
            $p(e) => e,
            _ => panic!("unexpected enum variant"),
        }
    };
}*/

/// The resource store type for all UI stuff (images, etc.)
pub type ResourceStore = Store<()>;

/// Various global UI states.
pub struct Ui {
    id_stack: IdStack,
    _cur_frame: u64,
    cursor_pos: (f32, f32),
    capture: Option<PointerCapture>,
    focus_path: Option<Vec<ItemID>>,
    stylesheets: Vec<Res<css::Stylesheet>>,
    roots: ItemChildren,
    cur_root_index: usize,
    frontmost_z: i32,
    store: ResourceStore,
    frame_index: usize
}

//
// Separate the UI tree from the visual tree.
//

impl Ui {
    /*fn add_popup(&mut self, id_path: &[ItemID])
    {

    }*/
    fn stylesheets_dirty(&self) -> bool {
        self.stylesheets
            .iter()
            .any(|s| s.borrow().dirty.replace(false))
    }

    fn set_focus(&mut self, path: Vec<ItemID>) {
        self.focus_path = Some(path);
    }

    fn release_focus(&mut self) {
        self.focus_path = None;
    }

    fn set_capture(&mut self, path: Vec<ItemID>) {
        debug!("set capture {:?}", &path[..]);
        self.capture = Some(PointerCapture {
            id_path: path,
            origin: self.cursor_pos,
        });
    }

    fn release_capture(&mut self) {
        debug!("release capture");
        self.capture = None;
    }

    /// Check if the given item is capturing pointer events.
    fn is_item_capturing(&self, id: ItemID) -> bool {
        if let Some(ref capture) = self.capture {
            *capture.id_path.last().expect("path was empty") == id
        } else {
            false
        }
    }

    /*fn hit_test_item_rec(
        &self,
        pos: (f32, f32),
        node: &ItemNode,
        path: &[ItemID],
        chain: &mut Vec<ItemID>,
    ) -> bool {
        if let Some((x, xs)) = path.split_first() {
            chain.push(node.item.id);
            self.hit_test_item_rec(pos, &node.children[x], xs, chain)
        } else {
            self.hit_test_rec(pos, node, chain)
        }
    }*/

    fn hit_test_rec(&self, pos: (f32, f32), node: &ItemNode, chain: &mut Vec<ItemID>) -> bool {
        if node.hit_test(pos) {
            chain.push(node.item.id);
            for (_, child) in node.children.0.iter() {
                if self.hit_test_rec(pos, child, chain) {
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    fn hit_test(
        &self,
        pos: (f32, f32),
        chain: &mut Vec<ItemID>,
    ) -> bool {
        // check popups first
        for (k,node) in self.roots.0.iter().rev() {
           // debug!("testing {}", *k);
            if self.hit_test_rec(pos, node, chain) {
                // got a match
                break;
            }
            chain.clear();
        }

        // a popup matched
        !chain.is_empty()
    }

    fn calculate_style(
        &mut self,
        node: &mut ItemNode,
        renderer: &Renderer,
        _parent: &CachedStyle,
        stylesheets_dirty: bool,
    ) {
        // TODO caching the full computed style in each individual item is super expensive (in CPU and memory)

        // recompute from stylesheet if classes have changed
        // TODO inherit
        if node.item.styles_dirty || stylesheets_dirty {
            //debug!("Full style calculation");
            // initiate a full recalculation.
            // TODO inherit
            let mut style = ComputedStyle::default();
            for stylesheet in self.stylesheets.iter() {
                let stylesheet = stylesheet.borrow();
                if let Some(ref class) = node.item.css_class {
                    // 1. fetch all applicable rules
                    // TODO actually fetch all rules.
                    let class_rule = stylesheet.match_class(class);
                    //debug!("rule {:?}", class_rule);
                    if let Some(class_rule) = class_rule {
                        // apply rule
                        for d in class_rule.declarations.iter() {
                            style.apply_property(d);
                        }
                        //debug!("calculated layout for {}: {:#?}", first, style.layout);
                    }
                }
            }

            // update the cached style
            let layout_damaged = node.item.style.update(&style);
            node.item.styles_dirty = false;

            if layout_damaged {
                // must update flexbox properties of the layout tree
                //debug!("layout is damaged");
                apply_to_flex_node(&mut node.flexbox, &node.item.style);
            }
        }

        // apply layout overrides: they always have precedence over the computed styles
        let m = node.measure(renderer);
        m.width.map(|w| {
            node.flexbox.set_width(w.point());
        });
        m.height.map(|h| {
            node.flexbox.set_height(h.point());
        });
        node.item
            .layout_overrides
            .left
            .map(|v| node.flexbox.set_position(yoga::Edge::Left, v));
        node.item
            .layout_overrides
            .top
            .map(|v| node.flexbox.set_position(yoga::Edge::Top, v));
        node.item
            .layout_overrides
            .width
            .map(|v| node.flexbox.set_width(v));
        node.item
            .layout_overrides
            .height
            .map(|v| node.flexbox.set_height(v));

        for (_, child) in node.children.0.iter_mut() {
            self.calculate_style(child, renderer, &node.item.style, stylesheets_dirty);
        }
    }

    fn render_item(
        &mut self,
        node: &mut ItemNode,
        parent_layout: &Layout,
        draw_list: &mut DrawList,
    ) {
        let layout = Layout::from_yoga_layout(parent_layout, node.flexbox.get_layout());
        node.item.layout = layout;
        //debug!("layout {:?}", layout);
        draw_list.with_z_order(node.item.z_order, |draw_list| {
            node.draw(draw_list);
            for (_, child) in node.children.0.iter_mut() {
                self.render_item(child, &layout, draw_list);
            }
        });
    }
}

fn measure_time<F: FnOnce()>(f: F) -> u64 {
    let start = time::PreciseTime::now();
    f();
    let duration = start.to(time::PreciseTime::now());
    duration.num_microseconds().unwrap() as u64
}


impl Ui {
    /// Creates a new Ui object.
    pub fn new() -> Ui {
        // The root node of the main window (ID 0).
        let root = ItemNode::new(0, Box::new(()));
        // roots
        let mut roots = ItemChildren::new();
        roots.0.insert(0, root);

        let ui = Ui {
            id_stack: IdStack::new(0),
            _cur_frame: 0,
            cursor_pos: (0.0, 0.0),
            capture: None,
            focus_path: None,
            stylesheets: Vec::new(),
            roots,
            frontmost_z: -1,
            cur_root_index: 0,
            store: ResourceStore::new(StoreOpt::default()).expect("unable to create the store"),
            frame_index: 0
        };

        ui
    }

    /// Loads a CSS stylesheet from the specified path.
    pub fn load_stylesheet<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let mut ctx = ();
        let stylesheet = self
            .store
            .get::<_, Stylesheet>(&FSKey::new(path.as_ref().clone()), &mut ctx)?;
        debug!("loading stylesheet at {}", path.as_ref().display());
        self.stylesheets.push(stylesheet);
        Ok(())
    }

    /// Dispatches a `WindowEvent` to items.
    /// First, the function determines the chain of items that will receive the event
    /// (the `DispatchChain`).
    /// Then, the capture event handler for each item in the chain is called, in order,
    /// starting from root until the event is captured, or the item preceding the target is reached.
    /// (capture phase).
    /// If the event is not captured, then the bubble event handlers are called,
    /// in reverse order from the target to the root, until the event is captured (bubble phase).
    pub fn dispatch_event(&mut self, event: &WindowEvent) {
        let _event_dispatch_time = measure_time(|| {
            // update state
            match event {
                &WindowEvent::CursorMoved { position, .. } => {
                    self.cursor_pos = (position.0 as f32, position.1 as f32);
                }
                &WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button: _,
                    modifiers: _,
                } => {
                    if state == ElementState::Released {
                        // implicit capture release
                        debug!("implicit capture release");
                        self.release_capture();
                    }
                }
                _ => {}
            };

            // build dispatch chain
            let (dispatch_items, target) = if let Some(ref capture) = self.capture {
                (capture.id_path.clone(), DispatchTarget::Capture)
            } else if let Some(ref focus) = self.focus_path {
                (focus.clone(), DispatchTarget::Focus)
            } else {
                let mut hit_test_chain = Vec::new();
                self.hit_test(
                    self.cursor_pos,
                    &mut hit_test_chain,
                );
                (hit_test_chain, DispatchTarget::HitTest)
            };

            /*debug!("dispatch chain: ");
            for (i,id) in dispatch_items.iter().enumerate() {
                debug!("#{}({:016X})", i, id);
            }*/

            if !dispatch_items.is_empty() {
                let dispatch_chain = DispatchChain {
                    items: &dispatch_items[..],
                    target,
                    current: 0,
                };

                if let Some(root_id) = dispatch_chain.items.first() {
                    // extract root to avoid multiple mut borrows of self
                    let mut root = self
                        .roots.0.remove(root_id)
                        .expect("root node not found");
                    root.propagate_event(event, self, dispatch_chain.clone());
                    self.roots.0.insert(*root_id, root);
                }
            }
        });
    }

    /// TODO document.
    pub fn root<F: FnOnce(&mut UiContainer)>(&mut self, f: F) {
        let mut ctx = ();
        // this should probably be done in its own function.
        self.store.sync(&mut ctx);

        let spec_time = measure_time(|| {
            let mut window_root = self.roots.0.remove(&0).expect("main window root not found");
            {
                let mut ui = UiContainer {
                    ui: self,
                    children: &mut window_root.children,
                    flexbox: &mut window_root.flexbox,
                    id: 0,
                    cur_index: 0,
                };
                f(&mut ui);
                ui.finish();
            }
            self.roots.0.insert(0, window_root);
            self.sort_roots();
            self.frontmost_z = self.roots.0.values().last().unwrap().item.z_order.unwrap();
            self.frame_index += 1;
        });
    }

    fn calculate_layouts(&mut self, roots: &mut ItemChildren, size: (f32,f32))
    {
        for (k,v) in roots.0.iter_mut() {
            v.flexbox.calculate_layout(size.0, size.1, yoga::Direction::LTR);
        }
    }

    fn calculate_styles(&mut self, roots: &mut ItemChildren, renderer: &Renderer, root_style: &CachedStyle, stylesheets_dirty: bool)
    {
        for (k,v) in roots.0.iter_mut() {
            self.calculate_style(v, renderer, root_style, stylesheets_dirty);
        }
    }

    fn sort_roots(&mut self)
    {
        self.roots.0.sort_by(|ka, va, kb, vb| {
            // stable sort
            va.item.z_order.unwrap_or(-1).cmp(&vb.item.z_order.unwrap_or(-1))
        });
        // normalize orders
        for (i,v) in self.roots.0.values_mut().enumerate() {
            v.item.z_order = Some(i as i32);
        };
    }

    /// Renders the UI to the given renderer.
    /// This function first calculates the styles, then performs layout,
    /// and finally calls the draw() function of each ItemBehavior in the hierarchy.
    pub fn render(&mut self, size: (f32, f32), renderer: &mut Renderer) {
        // measure contents pass
        use ::std::mem::replace;
        let mut roots = replace(&mut self.roots, ItemChildren::new());

        let _style_calculation_time = measure_time(|| {
            let root_style = CachedStyle::default();
            // are the sheets dirty?
            let stylesheets_dirty = self.stylesheets_dirty();
            self.calculate_styles(&mut roots, renderer, &root_style, stylesheets_dirty);
        });
        let _layout_time = measure_time(|| {
            self.calculate_layouts(&mut roots, size);
        });
        let root_layout = Layout {
            left: 0.0,
            top: 0.0,
            right: size.0,
            bottom: size.1,
        };
        let _render_time = measure_time(|| {
            let mut draw_list = DrawList::new();
            for (k,v) in roots.0.iter_mut() {
                self.render_item(v, &root_layout, &mut draw_list);
            }
            draw_list.sort();
            renderer.draw_frame(&draw_list.items[..]);
        });

        replace(&mut self.roots, roots);
        //debug!("style {}us, layout {}us, render {}us", style_calculation_time, layout_time, render_time);
    }
}
