#[macro_use]
extern crate log;
extern crate winit;
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
extern crate webrender;
extern crate gleam;
extern crate euclid;

// modules
mod css;
mod id_stack;
mod input;
mod layout;
mod renderer;
mod style;
mod vdom;
mod component;
mod behavior;
mod panel;
mod id_tree;

// std uses
use std::path::{Path};
use std::collections::hash_map::HashMap;
use std::any::Any;

// external crate uses
use glutin::{GlWindow, EventsLoop};
use failure::Error;

// self uses
use self::input::{DispatchChain, DispatchTarget, PointerCapture};
use self::id_tree::*;
use self::renderer::{WebrenderContext, layout_and_render_dom};

// self re-exports
pub use self::component::*;
pub use self::vdom::*;
pub use self::panel::*;
pub use self::css::Stylesheet;
pub use self::id_stack::{IdStack, ElementID};
pub use self::input::InputState;
pub use self::layout::{ContentMeasurement, Layout};
pub use self::style::{
    Background, Color, Styles, LinearGradient, RadialGradient, StyleCache
};

// external re-exports
pub use glutin::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
pub use yoga::prelude::*;
pub use warmy::{FSKey, Res, Store, StoreOpt};


/// The resource store type for all UI stuff (images, etc.)
pub type ResourceStore = Store<()>;

/// Update the styles for this element from stylesheets.
fn update_styles(arena: &mut Arena<RetainedNode>,
                 id: NodeId,
                 stylesheets: &[Res<Stylesheet>],
                 styles_cache: &mut StyleCache,
                 //renderer: &Renderer,
                 force: bool)
{
    {
        let node = &mut arena[id];
        let data = node.data_mut();
        let dirty = data.styles_dirty;
        if dirty || force {
            let new_styles = styles_cache.get_styles(stylesheets, css::Selector::new(data.class.clone()));
            let layout_damaged = if let Some(ref mut styles) = data.styles {
                let layout_damaged = styles.layout != new_styles.layout;
                *styles = new_styles;
                layout_damaged
            } else {
                data.styles = Some(new_styles);
                true
            };

            if layout_damaged {
                style::apply_to_flex_node(&mut data.flex, data.styles.as_ref().unwrap());
            }
        }

        data.styles_dirty = false;

        if let Contents::Text(ref text) = data.contents {
            // measure text
            //renderer.measure_text(text, elt.extra.styles.as_ref().unwrap());
        }

        // apply layout overrides: they always have precedence over the computed styles
        /*let m = node.measure(renderer);
        m.width.map(|w| {
            node.flexbox.set_width(w.point());
        });
        m.height.map(|h| {
            node.flexbox.set_height(h.point());
        });*/

        data.layout_overrides
            .left
            .map(|v| data.flex.set_position(yoga::Edge::Left, v));
        data.layout_overrides
            .top
            .map(|v| data.flex.set_position(yoga::Edge::Top, v));
        data.layout_overrides
            .width
            .map(|v| data.flex.set_width(v));
        data.layout_overrides
            .height
            .map(|v| data.flex.set_height(v));

        // drop arena borrow
    }

    let mut next = arena[id].first_child();
    while let Some(id) = next {
        update_styles(arena, id, stylesheets, styles_cache, force);
        next = arena[id].next_sibling();
    }
}

/// All states
pub struct Ui
{
    components: HashMap<ElementID, Box<Any>>,
    id_stack: IdStack,
    _cur_frame: u64,
    cursor_pos: (f32, f32),
    capture: Option<PointerCapture>,
    stylesheets: Vec<Res<css::Stylesheet>>,
    style_cache: StyleCache,
    store: ResourceStore,
    dom_nodes: Arena<RetainedNode>,
    dom_root: Option<NodeId>,
    main_wr_context: WebrenderContext,
    /// Owned windows (created by the UI).
    side_windows: Vec<(GlWindow, WebrenderContext)>
}

impl Ui
{
    pub fn get_component<C, NewFn>(&mut self, id: ElementID, new_fn: NewFn) -> Box<C>
        where
            C: Component,
            NewFn: FnOnce() -> C
    {
        self.components.remove(&id).map(|c| c.downcast().expect("invalid component type")).unwrap_or_else(|| {
            let mut component = Box::new(new_fn());
            component
        })
    }

    pub fn insert_component<C>(&mut self, id: ElementID, component: Box<C>)
        where
            C: Component
    {
        self.components.insert(id, component);
    }

    fn stylesheets_dirty(&self) -> bool {
        self.stylesheets
            .iter()
            .any(|s| s.borrow().dirty.replace(false))
    }
}

pub fn measure_time<F: FnOnce()>(f: F) -> u64 {
    let start = time::PreciseTime::now();
    f();
    let duration = start.to(time::PreciseTime::now());
    duration.num_microseconds().unwrap() as u64
}

impl Ui {
    /// Creates a new Ui object.
    pub fn new(main_window: &glutin::GlWindow, events_loop: &glutin::EventsLoop) -> Ui {
        let ui = Ui {
            id_stack: IdStack::new(0),
            components: HashMap::new(),
            _cur_frame: 0,
            cursor_pos: (0.0, 0.0),
            capture: None,
            stylesheets: Vec::new(),
            style_cache: StyleCache::new(),
            store: ResourceStore::new(StoreOpt::default()).expect("unable to create the store"),
            dom_nodes: Arena::new(),
            dom_root: None,
            main_wr_context: WebrenderContext::new(main_window, events_loop),
            side_windows: Vec::new(),
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

    /// Renders the UI.
    pub fn render(&mut self, window: &glutin::GlWindow) {
        // reload resources if necessary.
        self.store.sync(&mut ());
        if let Some(dom_root) = self.dom_root {
            update_styles(&mut self.dom_nodes, dom_root, &self.stylesheets[..], &mut self.style_cache, false);
            layout_and_render_dom(window, &mut self.main_wr_context, &mut self.dom_nodes, dom_root);
        }
        // TODO: render side windows.
    }

    // issues with hit-testing:
    // - how to generate the propagation path?
    // - how to recover the RetainedElement?
    // ... use petgraph for the retainedDOM? and access nodes by ID.
    // ... or use an ID tree: https://github.com/maps4print/azul/blob/f141ce17c501c3fe8edd1db8a07428ae722a5c9e/src/id_tree.rs
    // => Hit-test, then follow chain of parents to build the propagation path.

/*
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
    }*/

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

    /*fn hit_test_rec(&self, pos: (f32, f32), node: &ItemNode, chain: &mut Vec<ItemID>) -> bool {
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
    }*/



    /*fn render_item(
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
    }*/

    /// Receives a window event.
    pub fn event(&mut self, event: &WindowEvent)
    {
    }

    /// Update the DOM with the provided VDOM data
    pub fn update(&mut self, f: impl FnOnce(&mut DomSink))
    {
        let roots = {
            let mut dom = DomSink::new(self);
            f(&mut dom);
            dom.into_nodes()
        };

        let vdom = VirtualNode::new_element(0, "root", roots);

        if let Some(dom_root) = self.dom_root {
            update_node(&mut self.dom_nodes, dom_root, vdom);
        } else {
            self.dom_root = Some(vdom.into_retained(&mut self.dom_nodes, None));
        }
    }
}
