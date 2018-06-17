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
    Background, CachedStyle, Color, ComputedStyle, LinearGradient, RadialGradient,
};

pub use glutin::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
pub use yoga::prelude::*;
pub use warmy::{FSKey, Res, Store, StoreOpt};

pub use self::panel::*;

use self::input::{DispatchChain, DispatchTarget, PointerCapture};
//use self::style::apply_to_flex_node;

/// The resource store type for all UI stuff (images, etc.)
pub type ResourceStore = Store<()>;

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

    /// Update the DOM with the provided VDOM data
    pub fn update(&mut self, vdom: VirtualElement)
    {

    }
}
