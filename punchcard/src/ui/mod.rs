// cassowary is too slow unfortunately: took 693ms to layout 100 items in a vbox
// good for static layouts, not so much for imgui context with a highly dynamic item count
// use yoga instead
use diff;
use indexmap::{map::Entry, map::OccupiedEntry, map::VacantEntry, IndexMap};
use nvg;
use std::any::Any;
use std::fs;
use std::cell::{Cell, RefCell};
use std::collections::{hash_map, HashMap};
use std::path::{Path,PathBuf};
use std::hash::{Hash, Hasher, SipHasher};
use std::marker::PhantomData;
use std::mem;
use time;
use yoga;
use yoga::prelude::*;
use yoga::FlexStyle::*;
use yoga::StyleUnit::{Auto, UndefinedValue};
use failure::Error;

// Top priority:
// - DONE deferred event propagation (with capture and bubble stages)
// - WIP split into modules
//      mod.rs(->InputState,Ui), renderer, layout, style, item(event,input_state), container(ui_state)
// - rework style!() macro
// - default draw() callback (it's mostly the same each time)
// - DONE alternative callbacks (|ui,item,state| : more consistent with other callbacks, but more parameters)
// - buttons
// - DONE sliders
// - checkboxes
// - DONE hbox layout
// - native window handling
// - WIP style computation

// Alternative:
// take closure with: |ui,item,state|
// - ui handles adding children
// - item is the actual item
// - state is the item state (typed)

// Proposing a more radical change:
// - now: all callbacks have a 'static bound (draw, input, measure)
// - replace with a single trait, that encapsulates the widget state and deferred behavior
// - trait ItemBehavior (draw, input, measure)
// - can impl a trait inside a function
// - ItemNode = Item(layout, styles, etc.) + Behavior(draw, input, measure)
// - Behavior contains internal state, knows the type
// - UiContainer: &mut behavior (correct type), &mut Item (styles, etc), flexbox, children (invisible)
//          -> both are disjoint
// - maybe harder to create one-shot behaviors?
//          -> compensate by improving the immediate path

mod container;
mod item;
mod layout;
mod renderer;
mod style;
mod css;

// Reexports
pub use self::container::{ScrollState, UiContainer};
use self::item::ItemNode;
pub use self::item::{DummyBehavior, Item, ItemBehavior};
pub use self::layout::{ContentMeasurement, Layout};
pub use self::renderer::{ImageCache, NvgRenderer, Renderer};
pub use self::style::{Background, Color, LinearGradient, RadialGradient, Style};
pub use glutin::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

type ItemID = u64;

macro_rules! unwrap_enum {
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
}

/// The ID stack. Each level corresponds to a parent ItemNode.
pub struct IdStack(Vec<ItemID>);

impl IdStack {
    /// Creates a new IdStack and push the specified ID onto it.
    pub fn new(root_id: ItemID) -> IdStack {
        IdStack(vec![root_id])
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> ItemID {
        let stacklen = self.0.len();
        let key1 = if stacklen >= 2 {
            self.0[stacklen - 2]
        } else {
            0
        };
        let key0 = if stacklen >= 1 {
            self.0[stacklen - 1]
        } else {
            0
        };
        let mut sip = SipHasher::new_with_keys(key0, key1);
        s.hash(&mut sip);
        sip.finish()
    }

    /// Hashes the given data, initializing the hasher with the items currently on the stack.
    /// Pushes the result on the stack and returns it.
    /// This is used to generate a unique ID per item path in the hierarchy.
    pub fn push_id<H: Hash>(&mut self, s: &H) -> ItemID {
        let id = self.chain_hash(s);
        let parent_id = *self.0.last().unwrap();
        self.0.push(id);
        id
    }

    /// Pops the ID at the top of the stack.
    pub fn pop_id(&mut self) {
        self.0.pop();
    }
}

/// Struct containing information about a pointer capture.
#[derive(Clone)]
pub struct PointerCapture {
    /// Where the mouse button was at capture.
    origin: (f32, f32),
    /// The path (hierarchy of IDs) to the element that is capturing the mouse pointer.
    id_path: Vec<ItemID>,
}

/// Describes the nature of the target of a dispatch chain.
#[derive(Copy, Clone, Debug)]
enum DispatchTarget {
    /// The dispatch chain targets a captured item.
    Capture,
    /// The dispatch chain targets a focused item.
    Focus,
    /// The dispatch chain targets a leaf item that passed the cursor hit-test.
    HitTest,
}

/// Represent a dispatch chain: a chain of items that should receive an event.
#[derive(Copy, Clone)]
struct DispatchChain<'a> {
    /// the items in the chain
    items: &'a [ItemID],
    /// current position in the chain
    current: usize,
    /// reason for dispatch
    target: DispatchTarget,
}

impl<'a> DispatchChain<'a> {
    /// advance position in the chain
    fn next(&self) -> Option<DispatchChain<'a>> {
        if self.current + 1 < self.items.len() {
            Some(DispatchChain {
                items: self.items,
                current: self.current + 1,
                target: self.target,
            })
        } else {
            None
        }
    }

    /// Get the current item ID
    fn current_id(&self) -> ItemID {
        self.items[self.current]
    }

    /// Returns the final target of this dispatch chain.
    fn target_id(&self) -> ItemID {
        *self.items.last().unwrap()
    }

    /// Returns the currently processed chain, including the current element.
    fn current_chain(&self) -> &'a [ItemID] {
        &self.items[0..=self.current]
    }
}

/// Struct passed to event handlers.
pub struct InputState<'a> {
    /// TODO document
    state: &'a mut UiState,
    /// Dispatch chain that the event is travelling along.
    dispatch_chain: DispatchChain<'a>,
    /// Whether the item received this event because it has been captured.
    capturing: bool,
    /// Whether the item received this event because it has focus.
    focused: bool,
}

impl<'a> InputState<'a> {
    /// Signals that the current item in the dispatch chain should capture all events.
    pub fn set_capture(&mut self) {
        self.state
            .set_capture(self.dispatch_chain.current_chain().into());
    }

    /// Signals that the current item should have focus.
    pub fn set_focus(&mut self) {
        self.state
            .set_focus(self.dispatch_chain.current_chain().into());
    }

    /// Get the pointer capture origin position.
    pub fn get_capture_origin(&self) -> Option<(f32, f32)> {
        self.state.capture.as_ref().map(|params| params.origin)
    }

    /// Get drag delta from start of capture.
    pub fn get_capture_drag_delta(&self) -> Option<(f32, f32)> {
        self.state.capture.as_ref().map(|params| {
            let (ox, oy) = params.origin;
            let (cx, cy) = self.state.cursor_pos;
            (cx - ox, cy - oy)
        })
    }

    /// Release the capture. This fails (silently) if the current item is not
    /// capturing events.
    pub fn release_capture(&mut self) {
        // check that we are capturing
        if self.capturing {
            self.state.release_capture()
        } else {
            warn!("trying to release capture without capturing");
        }
    }

    /// Get the current cursor position.
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.state.cursor_pos
    }
}

/// Various global UI states.
pub struct UiState {
    id_stack: IdStack,
    cur_frame: u64,
    cursor_pos: (f32, f32),
    capture: Option<PointerCapture>,
    focus_path: Option<Vec<ItemID>>,
    stylesheets: Vec<css::Stylesheet>
}

impl UiState {
    fn new() -> UiState {
        UiState {
            id_stack: IdStack::new(0),
            cur_frame: 0,
            cursor_pos: (0.0, 0.0),
            capture: None,
            focus_path: None,
            stylesheets: Vec::new()
        }
    }

    fn load_stylesheet<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error>
    {
        let src = fs::read_to_string(path)?;
        let stylesheet = css::parse_stylesheet(&src)?;
        debug!("loaded stylesheet: {:#?}", stylesheet);
        self.stylesheets.push(stylesheet);
        Ok(())
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

    fn hit_test(&self, pos: (f32, f32), node: &ItemNode, chain: &mut Vec<ItemID>) -> bool {
        if node.hit_test(pos) {
            chain.push(node.item.id);
            for (_, child) in node.children.iter() {
                if self.hit_test(pos, child, chain) {
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    fn dispatch_event(&mut self, root_node: &mut ItemNode, event: &WindowEvent) {
        // update state
        match event {
            &WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.0 as f32, position.1 as f32);
            }
            &WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
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
            // TODO hit-test
            let mut hit_test_chain = Vec::new();
            self.hit_test(self.cursor_pos, root_node, &mut hit_test_chain);
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

    fn calculate_style(&mut self, node: &mut ItemNode, renderer: &Renderer, parent: &Style) {
        let style = node.item.style.inherit(parent).with_default(parent);
        node.item.calculated_style = style.clone();
        // measure item
        let m = node.behavior.measure(&mut node.item, renderer);
        if let Some(width) = m.width {
            style!(node.flexbox, Width(width.point()))
        }
        if let Some(height) = m.height {
            style!(node.flexbox, Height(height.point()))
        }

        for (_, child) in node.children.iter_mut() {
            self.calculate_style(child, renderer, &style);
        }
    }

    fn render_item(
        &mut self,
        node: &mut ItemNode,
        parent_layout: &Layout,
        renderer: &mut Renderer,
    ) {
        let layout = Layout::from_yoga_layout(parent_layout, node.flexbox.get_layout());
        node.item.layout = layout;
        node.behavior.draw(&mut node.item, renderer);
        for (_, child) in node.children.iter_mut() {
            self.render_item(child, &layout, renderer);
        }
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
    root: ItemNode,
    state: UiState,
}

impl Ui {
    /// Creates a new Ui object.
    pub fn new() -> Ui {
        let mut root = ItemNode::new(0, Box::new(DummyBehavior));

        let mut ui = Ui {
            root,
            state: UiState::new(),
        };
        ui
    }

    /// Loads a CSS stylesheet from the specified path.
    pub fn load_stylesheet<P: AsRef<Path>>(&mut self, path: P) -> Result<(),Error> {
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
        let event_dispatch_time = measure_time(|| {
            self.state.dispatch_event(&mut self.root, event);
        });
        //debug!("event dispatch took {}us", event_dispatch_time);
    }

    /// TODO document.
    pub fn root<F: FnOnce(&mut UiContainer)>(&mut self, f: F) {
        let spec_time = measure_time(|| {
            let mut ui = UiContainer::new_root(0, &mut self.root, &mut self.state);
            f(&mut ui);
            ui.finish()
        });
        //debug!("ui specification took {}us", spec_time);
    }

    /// Renders the UI to the given renderer.
    /// This function first calculates the styles, then performs layout,
    /// and finally calls the draw() function of each ItemBehavior in the hierarchy.
    pub fn render(&mut self, size: (f32, f32), renderer: &mut Renderer) {
        // measure contents pass
        let style_calculation_time = measure_time(|| {
            let root_style = Style::empty();
            self.state
                .calculate_style(&mut self.root, renderer, &root_style);
        });
        let layout_time = measure_time(|| {
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
        let render_time = measure_time(|| {
            self.state
                .render_item(&mut self.root, &root_layout, renderer);
        });

        // debug!("style {}us, layout {}us, render {}us", style_calculation_time, layout_time, render_time);
    }
}
