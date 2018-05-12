// cassowary is too slow unfortunately: took 693ms to layout 100 items in a vbox
// good for static layouts, not so much for imgui context with a highly dynamic item count
// use yoga instead
use diff;
use indexmap::{map::Entry, map::OccupiedEntry, map::VacantEntry, IndexMap};
use nvg;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{hash_map, HashMap};
use std::hash::{Hash, Hasher, SipHasher};
use std::marker::PhantomData;
use std::mem;
use time;
use yoga;
use yoga::prelude::*;
use yoga::FlexStyle::*;
use yoga::StyleUnit::{Auto, UndefinedValue};

// Top priority:
// - DONE deferred event propagation (with capture and bubble stages)
// - WIP split into modules
//      mod.rs(->InputState,Ui), renderer, layout, style, item(event,input_state), container(ui_state)
// - rework style!() macro
// - default draw() callback (it's mostly the same each time)
// - alternative callbacks (|ui,item,state| : more consistent with other callbacks, but more parameters)
// - buttons
// - DONE sliders
// - checkboxes
// - hbox layout
// - native window handling
// - WIP style computation

// Alternative:
// take closure with: |ui,item,state|
// - ui handles adding children
// - item is the actual item
// - state is the item state (typed)

mod container;
mod item;
mod layout;
mod renderer;
mod style;

// Reexports
pub use self::container::{ScrollState, UiContainer};
pub use self::item::Item;
pub use self::layout::{ContentMeasurement, Layout};
pub use self::renderer::{NvgRenderer, Renderer, ImageCache};
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

// Q: determination of the event propagation path:
// Mouse (no capture): from root to element that passes the hit-test
// Mouse (with capture): from root to element that captures pointer input
// Keyboard: from root to element with focus
//
// Q: how to set the focus?

pub struct IdStack(Vec<ItemID>);

impl IdStack {
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

    pub fn push_id<H: Hash>(&mut self, s: &H) -> ItemID {
        let id = self.chain_hash(s);
        let parent_id = *self.0.last().unwrap();
        self.0.push(id);
        id
    }

    pub fn pop_id(&mut self) {
        self.0.pop();
    }
}

#[derive(Clone)]
pub struct PointerCapture {
    /// Where the mouse button was at capture.
    origin: (f32, f32),
    /// The path (hierarchy of IDs) to the element that is capturing the mouse pointer.
    id_path: Vec<ItemID>,
}

///
#[derive(Copy, Clone, Debug)]
enum DispatchReason {
    Captured,
    Focused,
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
    reason: DispatchReason,
}

impl<'a> DispatchChain<'a> {
    /// advance position in the chain
    fn next(&self) -> Option<DispatchChain<'a>> {
        if self.current + 1 < self.items.len() {
            Some(DispatchChain {
                items: self.items,
                current: self.current + 1,
                reason: self.reason,
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
    state: &'a mut UiState,
    /// dispatch chain
    dispatch_chain: DispatchChain<'a>,
    /// Whether the item received this event because it has been captured
    capturing: bool,
    /// Whether the item received this event because it has focus
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
    /// capturing events
    pub fn release_capture(&mut self) {
        // check that we are capturing
        if self.capturing {
            self.state.release_capture()
        } else {
            warn!("trying to release capture without capturing");
        }
    }

    /// Get the current cursor position
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.state.cursor_pos
    }
}

pub struct UiState {
    id_stack: IdStack,
    cur_frame: u64,
    cursor_pos: (f32, f32),
    capture: Option<PointerCapture>,
    focus_path: Option<Vec<ItemID>>,
}

impl UiState {
    pub fn new() -> UiState {
        UiState {
            id_stack: IdStack::new(0),
            cur_frame: 0,
            cursor_pos: (0.0, 0.0),
            capture: None,
            focus_path: None,
        }
    }

    pub fn set_focus(&mut self, path: Vec<ItemID>) {
        self.focus_path = Some(path);
    }

    pub fn release_focus(&mut self) {
        self.focus_path = None;
    }

    pub fn set_capture(&mut self, path: Vec<ItemID>) {
        debug!("set capture {:?}", &path[..]);
        self.capture = Some(PointerCapture {
            id_path: path,
            origin: self.cursor_pos,
        });
    }

    pub fn release_capture(&mut self) {
        debug!("release capture");
        self.capture = None;
    }

    /// Check if the given item is capturing pointer events.
    pub fn is_item_capturing(&self, id: ItemID) -> bool {
        if let Some(ref capture) = self.capture {
            *capture.id_path.last().expect("path was empty") == id
        } else {
            false
        }
    }

    pub fn hit_test(
        &self,
        pos: (f32, f32),
        item_id: ItemID,
        item: &Item,
        chain: &mut Vec<ItemID>,
    ) -> bool {
        if item.hit_test(pos) {
            chain.push(item_id);
            for (&id, child) in item.children.iter() {
                if self.hit_test(pos, id, child, chain) {
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn dispatch_event(
        &mut self,
        root_item_id: ItemID,
        root_item: &mut Item,
        event: &WindowEvent,
    ) {
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
        let (dispatch_items, reason) = if let Some(ref capture) = self.capture {
            (capture.id_path.clone(), DispatchReason::Captured)
        } else if let Some(ref focus) = self.focus_path {
            (focus.clone(), DispatchReason::Focused)
        } else {
            // TODO hit-test
            let mut hit_test_chain = Vec::new();
            self.hit_test(
                self.cursor_pos,
                root_item_id,
                root_item,
                &mut hit_test_chain,
            );
            (hit_test_chain, DispatchReason::HitTest)
        };

        /*debug!("dispatch chain: ");
        for (i,id) in dispatch_items.iter().enumerate() {
            debug!("#{}({:016X})", i, id);
        }*/

        if !dispatch_items.is_empty() {
            let dispatch_chain = DispatchChain {
                items: &dispatch_items[..],
                reason,
                current: 0,
            };
            root_item.propagate_event(event, self, dispatch_chain);
        }
    }

    pub fn calculate_style(
        &mut self,
        id: ItemID,
        item: &mut Item,
        renderer: &Renderer,
        parent: &Style,
    ) {
        // TODO
        let style = item.style.inherit(parent).with_default(parent);
        item.calculated_style = style.clone();
        // measure item
        let m = item.measure(renderer);
        if let Some(width) = m.width {
            style!(item.flexbox, Width(width.point()))
        }
        if let Some(height) = m.height {
            style!(item.flexbox, Height(height.point()))
        }

        for (&id, child) in item.children.iter_mut() {
            self.calculate_style(id, child, renderer, &style);
        }
    }

    pub fn render_item(
        &mut self,
        id: ItemID,
        item: &mut Item,
        parent_layout: &Layout,
        renderer: &mut Renderer,
    ) {
        let flex_layout = item.flexbox.get_layout();
        let layout = Layout::from_yoga_layout(parent_layout, item.flexbox.get_layout());
        item.layout = layout;

        item.draw(renderer);

        for (&id, child) in item.children.iter_mut() {
            self.render_item(id, child, &layout, renderer);
        }
    }
}

fn measure_time<F: FnOnce()>(f: F) -> u64 {
    let start = time::PreciseTime::now();
    f();
    let duration = start.to(time::PreciseTime::now());
    duration.num_microseconds().unwrap() as u64
}

pub struct Ui {
    root: Item,
    state: UiState,
}

impl Ui {
    pub fn new() -> Ui {
        let mut root = Item::new();
        root.state = Cell::new(Some(Box::<()>::new(())));

        let mut ui = Ui {
            root,
            state: UiState::new(),
        };
        ui
    }

    pub fn dispatch_event(&mut self, event: &WindowEvent) {
        let event_dispatch_time = measure_time(|| {
            self.state.dispatch_event(0, &mut self.root, event);
        });
        //debug!("event dispatch took {}us", event_dispatch_time);
    }

    pub fn root<F: FnOnce(&mut UiContainer<()>)>(&mut self, f: F) {
        let spec_time = measure_time(|| {
            let mut ui = UiContainer::new_root(0, &mut self.root, &mut self.state);
            f(&mut ui);
            ui.finish()
        });
        //debug!("ui specification took {}us", spec_time);
    }

    pub fn render(&mut self, size: (f32, f32), renderer: &mut Renderer) {
        // measure contents pass
        let style_calculation_time = measure_time(|| {
            let root_style = Style::empty();
            self.state
                .calculate_style(0, &mut self.root, renderer, &root_style);
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
                .render_item(0, &mut self.root, &root_layout, renderer);
        });

        // debug!("style {}us, layout {}us, render {}us", style_calculation_time, layout_time, render_time);
    }
}
