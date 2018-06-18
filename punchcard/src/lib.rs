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
use std::collections::hash_map::HashMap;
use std::any::Any;

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

pub use self::component::*;
pub use self::vdom::*;
pub use self::css::Stylesheet;
pub use self::id_stack::{IdStack, ElementID};
pub use self::input::InputState;
pub use self::layout::{ContentMeasurement, Layout};
pub use self::renderer::{DrawItem, DrawItemKind, DrawList, Renderer};
pub use self::style::{
    Background, Color, Styles, LinearGradient, RadialGradient, StyleCache
};

pub use glutin::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
pub use yoga::prelude::*;
pub use warmy::{FSKey, Res, Store, StoreOpt};

pub use self::panel::*;

use self::input::{DispatchChain, DispatchTarget, PointerCapture};
//use self::style::apply_to_flex_node;

/// The resource store type for all UI stuff (images, etc.)
pub type ResourceStore = Store<()>;


/// Update the styles for this element from stylesheets.
fn update_styles(elt: &mut RetainedElement,
                            stylesheets: &[Res<Stylesheet>],
                            styles_cache: &mut StyleCache,
                            renderer: &Renderer,
                            force: bool)
{
    let dirty = elt.extra.styles_dirty;
    if dirty || force {
        let new_styles = styles_cache.get_styles(stylesheets, css::Selector::new(elt.class.clone()));
        let layout_damaged = if let Some(ref mut styles) = elt.extra.styles {
            let layout_damaged = styles.layout != new_styles.layout;
            *styles = new_styles;
            layout_damaged
        }
            else {
                elt.extra.styles = Some(new_styles);
                true
            };

        if layout_damaged {
            style::apply_to_flex_node(&mut elt.extra.flex, elt.extra.styles.as_ref().unwrap());
        }
    }

    elt.extra.styles_dirty = false;

    if let Contents::Text(ref text) = elt.contents {
        // measure text
        renderer.measure_text(text, elt.extra.styles.as_ref().unwrap());
    }

    // apply layout overrides: they always have precedence over the computed styles
    /*let m = node.measure(renderer);
    m.width.map(|w| {
        node.flexbox.set_width(w.point());
    });
    m.height.map(|h| {
        node.flexbox.set_height(h.point());
    });*/

    elt.layout_overrides
        .left
        .map(|v| elt.extra.flex.set_position(yoga::Edge::Left, v));
    elt.layout_overrides
        .top
        .map(|v| elt.extra.flex.set_position(yoga::Edge::Top, v));
    elt.layout_overrides
        .width
        .map(|v| elt.extra.flex.set_width(v));
    elt.layout_overrides
        .height
        .map(|v| elt.extra.flex.set_height(v));

    if let Contents::Div(ref mut children) = elt.contents {
        for child in children.iter_mut() {
            update_styles(child, stylesheets, styles_cache, renderer, force);
        }
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
    store: ResourceStore,
    dom: Option<RetainedElement>,
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
    pub fn new() -> Ui {
        let ui = Ui {
            id_stack: IdStack::new(0),
            components: HashMap::new(),
            _cur_frame: 0,
            cursor_pos: (0.0, 0.0),
            capture: None,
            stylesheets: Vec::new(),
            store: ResourceStore::new(StoreOpt::default()).expect("unable to create the store"),
            dom: None
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


    /// Update the DOM with the provided VDOM data
    pub fn update(&mut self, f: impl FnOnce(&mut DomSink))
    {
        let roots = {
            let mut dom = DomSink::new(self);
            f(&mut dom);
            dom.into_elements()
        };

        let vdom = VirtualElement::new_div(0, "root", roots);

        if let Some(ref mut dom) = self.dom {
            dom.update(vdom);
        } else {
            self.dom = Some(vdom.into_retained());
        }
    }
}
