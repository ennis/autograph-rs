use super::layout::{ContentMeasurement, Layout};
use super::renderer::Renderer;
use super::style::Style;
use super::{DispatchChain, DispatchReason, InputState, ItemID, UiState};

use glutin::{KeyboardInput, MouseButton, MouseScrollDelta, WindowEvent};
use indexmap::IndexMap;
use yoga;

use std::any::Any;
use std::cell::Cell;
use std::mem;

/// A widget.
pub struct Item {
    pub(super) flexbox: yoga::Node,
    pub layout: Layout,
    pub(super) children: IndexMap<ItemID, Item>,
    /// Callback that allows an item to capture an event in its propagation path.
    //capture_event: Option<Box<Fn(&mut Item, &InputState)>>,
    /// Custom state (default is Box<()>).
    /// The value is wrapped in cell to allow temporarily moving the state out
    /// and to avoid mut-borrowing the whole item.
    pub(super) state: Cell<Option<Box<Any>>>,
    /// Non-layout styles associated to this widget
    pub style: Style,
    /// Cached calculated styles
    pub calculated_style: Style,
    /// Optional callback for measuring the content
    pub(super) measure: Option<Box<Fn(&mut Item, &Renderer) -> ContentMeasurement>>,
    /// The draw callback, to draw stuff
    pub(super) draw: Option<Box<Fn(&mut Item, &mut Renderer)>>,
    /// is the mouse hovering the element?
    pub(super) hovering: bool,
    /// is the item capturing pointer events?
    pub(super) capturing_pointer: bool,
    /// Callback to handle events.
    pub(super) capture_event_handler:
        Option<Box<Fn(&mut Item, &WindowEvent, &mut InputState) -> bool>>,
    /// Callback to handle events.
    pub(super) event_handler: Option<Box<Fn(&mut Item, &WindowEvent, &mut InputState) -> bool>>,
}

// event dispatch:
// 1. build route
//    1.1. if capture in progress, get route from capture
//    1.2. if focus, and event is not a pointer event, get route from focus
//    1.3. otherwise, perform hit-test to get route to hit-test target
// 2. dispatch event to all items along the route
//
// Q: how to specify that the item has focus?
//   -> input_state.set_focus() // set focus on current item
//   -> input_state.set_capture()

impl Item {
    pub fn new() -> Item {
        Item {
            children: IndexMap::new(),
            flexbox: yoga::Node::new(),
            state: Cell::new(None),
            layout: Layout::default(),
            style: Style::empty(),
            calculated_style: Style::empty(),
            measure: None,
            draw: None,
            hovering: false,
            capturing_pointer: true,
            capture_event_handler: None,
            event_handler: None,
        }
    }

    pub fn with_measure<F: Fn(&mut Item, &Renderer) -> ContentMeasurement + 'static>(
        &mut self,
        f: F,
    ) {
        self.measure = Some(Box::new(f));
    }

    pub fn init_state<D: Any>(&mut self, default: D) -> &mut D {
        if self.state.get_mut().is_none() {
            self.state.replace(Some(Box::new(default)));
        }
        self.state
            .get_mut()
            .as_mut()
            .unwrap()
            .downcast_mut()
            .expect("wrong custom data type")
    }

    pub fn extract_state<State: 'static>(&mut self) -> Box<State> {
        self.state
            .take()
            .expect("state was empty")
            .downcast()
            .expect("unexpected state type")
    }

    pub fn replace_state<State: 'static>(&mut self, s: Box<State>) {
        self.state.replace(Some(s));
    }

    pub fn with_extract_state<State: 'static, R, F: FnMut(&mut Item, &mut State) -> R>(
        &mut self,
        mut f: F,
    ) -> R {
        let mut state = self.extract_state();
        let result = f(self, &mut *state);
        self.replace_state(state);
        result
    }

    /*pub fn get_custom_data<D: Any>(&self) -> &D {
        self.custom_data.as_ref().unwrap().downcast_ref::<D>().unwrap()
    }

    pub fn get_custom_data_mut<D: Any>(&mut self) -> &mut D {
        self.custom_data.as_mut().unwrap().downcast_mut::<D>().unwrap()
    }*/

    pub fn apply_styles<'b, I>(&mut self, styles: I)
    where
        I: IntoIterator<Item = &'b yoga::FlexStyle>,
    {
        self.flexbox.apply_styles(styles);
    }

    pub fn measure(&mut self, renderer: &Renderer) -> ContentMeasurement {
        // move the closure outside the item so we don't hold a borrow
        let measure = mem::replace(&mut self.measure, None);
        let result = if let Some(ref measure) = measure {
            measure(self, renderer)
        } else {
            ContentMeasurement {
                width: None,
                height: None,
            }
        };
        // move the closure back inside
        mem::replace(&mut self.measure, measure);
        result
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let draw = mem::replace(&mut self.draw, None);
        if let Some(ref draw) = draw {
            draw(self, renderer);
        }
        mem::replace(&mut self.draw, draw);
    }

    pub fn hit_test(&self, pos: (f32, f32)) -> bool {
        return self.layout.is_point_inside(pos);
    }

    fn handle_event(
        &mut self,
        event: &WindowEvent,
        state: &mut UiState,
        dispatch_chain: DispatchChain,
    ) -> bool {
        let handler = mem::replace(&mut self.event_handler, None);
        let consumed = if let Some(ref handler) = handler {
            let capturing = state.is_item_capturing(dispatch_chain.current_id());
            let mut input_state = InputState {
                state,
                dispatch_chain,
                capturing,
                focused: false,
            };
            handler(self, event, &mut input_state)
        } else {
            false
        };
        mem::replace(&mut self.event_handler, handler);
        consumed
    }

    fn capture_event(
        &mut self,
        event: &WindowEvent,
        state: &mut UiState,
        dispatch_chain: DispatchChain,
    ) -> bool {
        let handler = mem::replace(&mut self.capture_event_handler, None);
        let consumed = if let Some(ref handler) = handler {
            let capturing = state.is_item_capturing(dispatch_chain.current_id());
            let mut input_state = InputState {
                state,
                dispatch_chain,
                capturing,
                focused: false,
            };
            handler(self, event, &mut input_state)
        } else {
            false
        };
        mem::replace(&mut self.capture_event_handler, handler);
        consumed
    }

    /// Propagate an event (capture phase).
    pub(super) fn propagate_event(
        &mut self,
        event: &WindowEvent,
        state: &mut UiState,
        dispatch_chain: DispatchChain,
    ) -> bool {
        if self.capture_event(event, state, dispatch_chain) {
            // event was consumed, stop propagation
            return true;
        }

        // follow the rest of the path
        if let Some(next_dispatch) = dispatch_chain.next() {
            let consumed = {
                let next = self.children
                    .get_mut(&next_dispatch.current_id())
                    .expect("item deleted");
                next.propagate_event(event, state, next_dispatch)
            };

            if !consumed {
                // not consumed, bubble up
                self.handle_event(event, state, dispatch_chain)
            } else {
                // consumed
                true
            }
        } else {
            self.handle_event(event, state, dispatch_chain)
        }
    }
}
