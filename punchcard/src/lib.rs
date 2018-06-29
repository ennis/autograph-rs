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
//mod panel;
mod id_tree;
mod widgets;
mod prelude;

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
pub use self::input::{InputState, EventResult};
pub use self::layout::{ContentMeasurement, Layout};
pub use self::style::{
    Background, Color, Styles, LinearGradient, RadialGradient, StyleCache
};
pub use self::widgets::*;

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
    components: HashMap<ElementID, Box<ComponentAny>>,
    id_stack: IdStack,
    _cur_frame: u64,
    cursor_pos: (f32, f32),
    capture: Option<PointerCapture>,
    focus: Option<NodeId>,
    stylesheets: Vec<Res<css::Stylesheet>>,
    style_cache: StyleCache,
    store: ResourceStore,
    dom_nodes: Arena<RetainedNode>,
    dom_root: Option<NodeId>,
    main_wr_context: WebrenderContext,
    /// Owned windows (created by the UI).
    side_windows: Vec<(GlWindow, WebrenderContext)>,
    dump_next_event_dispatch_chain: bool
}

impl Ui
{
    pub fn get_component<C, NewFn>(&mut self, id: ElementID, new_fn: NewFn) -> Box<ComponentAny>
        where
            C: Component,
            NewFn: FnOnce() -> C
    {
        self.components.remove(&id).unwrap_or_else(|| {
            debug!("NEW COMPONENT");
            let mut component = Box::new(new_fn());
            component
        })
    }

    pub fn insert_component(&mut self, id: ElementID, component: Box<ComponentAny>)
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
            focus: None,
            stylesheets: Vec::new(),
            style_cache: StyleCache::new(),
            store: ResourceStore::new(StoreOpt::default()).expect("unable to create the store"),
            dom_nodes: Arena::new(),
            dom_root: None,
            main_wr_context: WebrenderContext::new(main_window, events_loop),
            side_windows: Vec::new(),
            dump_next_event_dispatch_chain: false
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
            let force_restyle = if self.stylesheets_dirty() {
                debug!("stylesheets dirty, forcing restyle");
                self.style_cache.invalidate();
                true
            } else {
                false
            };

            update_styles(&mut self.dom_nodes, dom_root, &self.stylesheets[..], &mut self.style_cache, force_restyle);
            layout_and_render_dom(window, &mut self.main_wr_context, &mut self.dom_nodes, dom_root);
        }
        // TODO: render side windows.
    }


    fn dump_dom_0(&self,
                id: NodeId,
                dispatch_chain: Option<&[NodeId]>,
                level: usize)
    {
        let node = &self.dom_nodes[id];
        let data = node.data();

        let is_in_dispatch_chain = if let Some(chain) = dispatch_chain {
            chain.contains(&id)
        } else { false };

        let is_component = self.components.get(&data.id).is_some();

        // + <type> nodeid class ElementId= Layout
        println!("{:indent$} {} {}{} {} {} {:016X} ({},{}:{}x{})", "",
                 if is_in_dispatch_chain { "✓" } else { "-" },
                 match data.contents {
                     Contents::Element => "div",
                     Contents::Text(_) => "text"
                 },
                 if is_component { "*" } else { "" },
                 id.as_u64(),
                 if data.class.is_empty() { "<empty>" } else { data.class.as_ref() },
                 data.id,
                 data.layout.left,
                 data.layout.top,
                 data.layout.width(),
                 data.layout.height(),
                 indent=level*2);

        let mut next = node.first_child();
        while let Some(id) = next {
            self.dump_dom_0(id, dispatch_chain, level+1);
            next = self.dom_nodes[id].next_sibling();
        }
    }

    fn dump_dom(&self, dispatch_chain: Option<&[NodeId]>)
    {
        if let Some(dom_root) = self.dom_root {
            self.dump_dom_0(dom_root, dispatch_chain, 0);
        }
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
    }*/

    fn build_dispatch_chain(&self, hit_id: NodeId) -> Vec<NodeId>
    {
        let mut node_ids = vec![hit_id];
        let mut current = hit_id;
        while let Some(id) = self.dom_nodes[current].parent() {
            node_ids.push(id);
            current = id;
        }
        node_ids.reverse();
        node_ids
    }

    fn handle_event_result(result: &EventResult, cursor_pos: (f32,f32), id: NodeId, capture: &mut Option<PointerCapture>, focus: &mut Option<NodeId>)
    {
        if result.set_capture {
            *capture = Some(PointerCapture {
                id,
                origin: cursor_pos
            });
        }
        if result.set_focus {
            *focus = Some(id);
        }
    }

    fn dispatch_event_0(&mut self, event: &WindowEvent, chain: &[NodeId], input_state: &InputState) -> bool
    {
        //debug!("dispatch[{:?}]", chain);
        let (&id,rest) = chain.split_first().expect("empty dispatch chain");

        let captured = {
            if !rest.is_empty() {
                // capture stage
                {
                    let node = &mut self.dom_nodes[id];
                    let data = node.data_mut();
                    if let Some(component) = self.components.get_mut(&data.id) {
                        // don't forget to set the target ID here, it's not set to anything meaningful?
                        let result = component.capture_event(data, event, input_state);
                        Self::handle_event_result(&result, self.cursor_pos, id, &mut self.capture, &mut self.focus);
                        // handle input capture
                        if result.stop_propagation {
                            return true
                        }
                    }
                }
                // dispatch further in the chain.
                self.dispatch_event_0(event, rest, input_state)
            } else {
                false
            }
        };


        return if !captured {
            // bubbled back up to us
            let node = &mut self.dom_nodes[id];
            let data = node.data_mut();
            // XXX must query the hash map again.
            // This shouldn't be needed, as the components cannot be added or
            // removed during event dispatch.
            // TODO use interior mutability primitives to fix this.
            if let Some(component) = self.components.get_mut(&data.id) {
                let result = component.event(data, event, input_state);
                Self::handle_event_result(&result, self.cursor_pos, id, &mut self.capture, &mut self.focus);
                result.stop_propagation
            } else {
                false
            }
        } else {
            true
        }
    }

    fn dispatch_event(&mut self, event: &WindowEvent, chain: &[NodeId])
    {
        if let Some(first) = chain.first() {
            let mut input_state = InputState {
                focused: false,
                capture: self.capture.clone(),
                cursor_pos: self.cursor_pos
            };
            self.dispatch_event_0(event, chain, &mut input_state);
            /*if let Some(ref capture) = self.capture {
                debug!("after event, node {:?} is capturing", capture.id);
            }*/
        }
    }

    fn hit_test(&self, pos: (f32, f32)) -> Vec<NodeId>
    {
        use webrender::api::*;
        // are we capturing?
        if let Some(ref capture) = self.capture {
            // yes, skip hit-test and send event directly to capture target.
            //debug!("capturing");
            self.build_dispatch_chain(capture.id)
        } else {
            // not capturing, perform hit-test
            let hits = renderer::hit_test(&self.main_wr_context, WorldPoint::new(pos.0, pos.1));
            if let Some(id) = hits.first() {
                self.build_dispatch_chain(*id)
            } else {
                Vec::new()
            }
        }
    }

    /// Receives a window event.
    pub fn event(&mut self, event: &WindowEvent)
    {
        match event {
            WindowEvent::CursorMoved { device_id, position, modifiers } => {
                // update cursor pos
                self.cursor_pos = (position.0 as f32, position.1 as f32);
            },
            &WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
            } => {
                if state == ElementState::Released {
                    // implicit capture release
                    debug!("implicit capture release");
                    self.capture = None;
                }
            }
            WindowEvent::MouseWheel { .. } => {},
            WindowEvent::KeyboardInput { device_id, input } => {
                match input.virtual_keycode {
                    Some(VirtualKeyCode::F12) => {
                        self.dump_dom(None);
                    },
                    Some(VirtualKeyCode::F11) => {
                        self.dump_next_event_dispatch_chain = true;
                    }
                    _ => {}
                };
                debug!("Keyboard input UNIMPLEMENTED {:?}", input);
            }
            _ => {}
        }

        match event {
            WindowEvent::CursorMoved { .. } |
            WindowEvent::MouseInput { .. } |
            WindowEvent::MouseWheel { .. } => {
                let dispatch_chain = self.hit_test(self.cursor_pos);
                if self.dump_next_event_dispatch_chain {
                    self.dump_dom(Some(&dispatch_chain[..]));
                    self.dump_next_event_dispatch_chain = false;
                }
                self.dispatch_event(event, &dispatch_chain[..]);
            },
            _ => {}
        }

    }

    /// Update the DOM with the provided VDOM data
    pub fn update(&mut self, f: impl FnOnce(&mut DomSink))
    {
        let roots = {
            let mut dom = DomSink::new(self);
            f(&mut dom);
            dom.into_nodes()
        };

        let id_stack_len = self.id_stack.0.len();
        assert!(id_stack_len == 1, "ID stack had incorrect length: {}", id_stack_len);

        let vdom = VirtualNode::new_element(0, "root", roots);

        if let Some(dom_root) = self.dom_root {
            update_node(&mut self.dom_nodes, dom_root, vdom);
        } else {
            self.dom_root = Some(vdom.into_retained(&mut self.dom_nodes, None));
        }
    }
}
