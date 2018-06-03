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
use self::item::ItemNode;
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
pub struct UiState {
    id_stack: IdStack,
    _cur_frame: u64,
    cursor_pos: (f32, f32),
    capture: Option<PointerCapture>,
    focus_path: Option<Vec<ItemID>>,
    stylesheets: Vec<Res<css::Stylesheet>>,
    /// Floating popup windows (transient).
    popups: Vec<Vec<ItemID>>,
    store: ResourceStore,
}

impl UiState {
    fn new() -> UiState {
        UiState {
            id_stack: IdStack::new(0),
            _cur_frame: 0,
            cursor_pos: (0.0, 0.0),
            capture: None,
            focus_path: None,
            stylesheets: Vec::new(),
            popups: Vec::new(),
            store: ResourceStore::new(StoreOpt::default()).expect("unable to create the store"),
        }
    }

    /*fn add_popup(&mut self, id_path: &[ItemID])
    {

    }*/

    fn load_stylesheet<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let mut ctx = ();
        let stylesheet = self
            .store
            .get::<_, Stylesheet>(&FSKey::new(path.as_ref().clone()), &mut ctx)?;
        debug!("loading stylesheet at {}", path.as_ref().display());
        self.stylesheets.push(stylesheet);
        Ok(())
    }

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

    fn hit_test_item_rec(
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
    }

    fn hit_test_rec(&self, pos: (f32, f32), node: &ItemNode, chain: &mut Vec<ItemID>) -> bool {
        if node.hit_test(pos) {
            chain.push(node.item.id);
            for (_, child) in node.children.iter() {
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
        node: &ItemNode,
        popups: &[Vec<ItemID>],
        chain: &mut Vec<ItemID>,
    ) -> bool {
        // check popups first
        for p in popups {
            if self.hit_test_item_rec(pos, node, &p[1..], chain) {
                // got a match
                break;
            }
            chain.clear();
        }

        if !chain.is_empty() {
            return true;
        }

        // check root
        self.hit_test_rec(pos, node, chain)
    }

    fn dispatch_event(&mut self, root_node: &mut ItemNode, event: &WindowEvent) {
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
                root_node,
                &self.popups[..],
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
            root_node.propagate_event(event, self, dispatch_chain);
        }
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
                // TODO more than one class.
                //debug!("css classes {:?}", node.item.css_classes);
                if let Some(first) = node.item.css_classes.first() {
                    // 1. fetch all applicable rules
                    // TODO actually fetch all rules.
                    let class_rule = stylesheet.match_class(first);
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

        for (_, child) in node.children.iter_mut() {
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
            for (_, child) in node.children.iter_mut() {
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

/// The UI.
/// Call root() to get a UiContainer that allows adding child items to the UI root.
pub struct Ui {
    /// Root node of the main window.
    root: ItemNode,
    state: UiState,
}

impl Ui {
    /// Creates a new Ui object.
    pub fn new() -> Ui {
        let root = ItemNode::new(0, Box::new(()));

        let ui = Ui {
            root,
            state: UiState::new(),
        };
        ui
    }

    /// Loads a CSS stylesheet from the specified path.
    pub fn load_stylesheet<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.state.load_stylesheet(path)
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
            self.state.dispatch_event(&mut self.root, event);
        });
    }

    /// TODO document.
    pub fn root<F: FnOnce(&mut UiContainer)>(&mut self, f: F) {
        let mut ctx = ();
        // this should probably be done in its own function.
        self.state.store.sync(&mut ctx);
        self.state.popups.clear();
        let spec_time = measure_time(|| {
            let mut ui = UiContainer::new_root(0, &mut self.root, &mut self.state);
            f(&mut ui);
            ui.finish()
        });
    }

    /// Renders the UI to the given renderer.
    /// This function first calculates the styles, then performs layout,
    /// and finally calls the draw() function of each ItemBehavior in the hierarchy.
    pub fn render(&mut self, size: (f32, f32), renderer: &mut Renderer) {
        // measure contents pass
        let _style_calculation_time = measure_time(|| {
            let root_style = CachedStyle::default();
            // are the sheets dirty?
            let stylesheets_dirty = self.state.stylesheets_dirty();
            self.state
                .calculate_style(&mut self.root, renderer, &root_style, stylesheets_dirty);
        });
        let _layout_time = measure_time(|| {
            self.root
                .flexbox
                .calculate_layout(size.0, size.1, yoga::Direction::LTR);
        });
        let root_layout = Layout {
            left: 0.0,
            top: 0.0,
            right: size.0,
            bottom: size.1,
        };
        let _render_time = measure_time(|| {
            let mut draw_list = DrawList::new();
            self.state
                .render_item(&mut self.root, &root_layout, &mut draw_list);
            draw_list.sort();
            renderer.draw_frame(&draw_list.items[..]);
        });

        //debug!("style {}us, layout {}us, render {}us", style_calculation_time, layout_time, render_time);
    }
}
