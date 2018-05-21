use super::layout::{ContentMeasurement, Layout};
use super::renderer::Renderer;
use super::style::ComputedStyle;
use super::{DispatchChain, DispatchTarget, InputState, ItemID, UiState};
use super::ResourceStore;
use super::css;

use glutin::{KeyboardInput, MouseButton, MouseScrollDelta, WindowEvent};
use indexmap::IndexMap;
use yoga;

use std::any::Any;
use std::cell::Cell;
use std::mem;

/// A set of callbacks that describes the behavior of an item for all deferred processing:
/// i.e., processing that happens outside the scope of the function calls that create or
/// update the item (the _immediate path_).
/// Typically, implementors of this trait are also used to store persistent internal state inside
/// items.
pub trait ItemBehavior: Any {
    /// One-time initialization.
    fn init(&mut self, item: &mut Item) {}

    /// Draw the item to the specified renderer.
    fn draw(&mut self, item: &mut Item, renderer: &mut Renderer) {
        renderer.draw_rect(&item.layout, &item.style);
    }

    /// Measure the given item using the specified renderer.
    fn measure(&mut self, item: &mut Item, renderer: &Renderer) -> ContentMeasurement {
        ContentMeasurement {
            width: None,
            height: None,
        }
    }

    /// Callback to handle an event passed to the item during the capturing phase.
    fn capture_event(
        &mut self,
        item: &mut Item,
        event: &WindowEvent,
        input_state: &mut InputState,
    ) -> bool {
        false
    }

    /// Callback to handle an event during bubbling phase.
    fn event(
        &mut self,
        item: &mut Item,
        event: &WindowEvent,
        input_state: &mut InputState,
    ) -> bool {
        false
    }
}

/// A wrapper around ItemBehavior and Any traits. See:
/// https://github.com/rust-lang/rfcs/issues/2035
/// https://stackoverflow.com/questions/26983355
pub(super) trait ItemBehaviorAny: ItemBehavior + Any {
    fn as_mut_behavior(&mut self) -> &mut ItemBehavior;
    fn as_mut_any(&mut self) -> &mut Any;
}

impl<T> ItemBehaviorAny for T
where
    T: ItemBehavior + Any,
{
    fn as_mut_behavior(&mut self) -> &mut ItemBehavior {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DummyBehavior;

impl ItemBehavior for DummyBehavior {
    fn draw(&mut self, _item: &mut Item, _renderer: &mut Renderer) {}

    fn measure(&mut self, _item: &mut Item, _renderer: &Renderer) -> ContentMeasurement {
        ContentMeasurement {
            width: None,
            height: None,
        }
    }

    fn capture_event(
        &mut self,
        item: &mut Item,
        event: &WindowEvent,
        input_state: &mut InputState,
    ) -> bool {
        // capture nothing
        false
    }

    /// Callback to handle an event during bubbling phase.
    fn event(
        &mut self,
        item: &mut Item,
        event: &WindowEvent,
        input_state: &mut InputState,
    ) -> bool {
        // always bubble
        false
    }
}

/// Represents a node in the item hierarchy.
pub(super) struct ItemNode {
    /// The corresponding node in the flexbox layout hierarchy.
    pub(super) flexbox: yoga::Node,
    /// Child nodes of this item.
    pub(super) children: IndexMap<ItemID, ItemNode>,
    /// A set of callbacks describing the behavior of the item during deferred processing.
    /// Each widget has its own implementation of ItemBehavior that also stores
    /// internal state specific to the widget.
    /// See `ItemBehavior` for more information.
    pub(super) behavior: Box<ItemBehaviorAny>,
    /// User-facing properties of the item.
    pub(super) item: Item,
}

impl ItemNode {
    pub fn new(id: ItemID, behavior: Box<ItemBehaviorAny>) -> ItemNode {
        let mut n = ItemNode {
            children: IndexMap::new(),
            flexbox: yoga::Node::new(),
            behavior,
            item: Item::new(id),
        };
        n.behavior.init(&mut n.item);
        n
    }

    pub fn measure(&mut self, renderer: &Renderer) -> ContentMeasurement {
        self.behavior.measure(&mut self.item, renderer)
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.behavior.draw(&mut self.item, renderer)
    }

    pub fn hit_test(&self, pos: (f32, f32)) -> bool {
        self.item.layout.is_point_inside(pos)
    }

    fn handle_event(
        &mut self,
        event: &WindowEvent,
        state: &mut UiState,
        dispatch_chain: DispatchChain,
    ) -> bool {
        let capturing = state.is_item_capturing(dispatch_chain.current_id());
        let mut input_state = InputState {
            state,
            dispatch_chain,
            capturing,
            focused: false,
        };
        self.behavior.event(&mut self.item, event, &mut input_state)
    }

    fn capture_event(
        &mut self,
        event: &WindowEvent,
        state: &mut UiState,
        dispatch_chain: DispatchChain,
    ) -> bool {
        let capturing = state.is_item_capturing(dispatch_chain.current_id());
        let mut input_state = InputState {
            state,
            dispatch_chain,
            capturing,
            focused: false,
        };
        self.behavior.event(&mut self.item, event, &mut input_state)
    }

    /// Propagate an event.
    pub(super) fn propagate_event(
        &mut self,
        event: &WindowEvent,
        state: &mut UiState,
        dispatch_chain: DispatchChain,
    ) -> bool {
        // capture stage.
        if self.capture_event(event, state, dispatch_chain) {
            // event was consumed, stop propagation
            return true;
        }

        // pass the event down the chain, and handle it if it bubbles up.
        if let Some(next_dispatch) = dispatch_chain.next() {
            let consumed = {
                // TODO: verify that the dispatch chain is still valid before propagating an event.
                // The dispatch chain of the focused (or capturing) item is invalid if an item of
                // the chain is deleted between the time of the creation of chain and the time of
                // event propagation.
                let next = self.children
                    .get_mut(&next_dispatch.current_id())
                    .expect("item deleted");
                next.propagate_event(event, state, next_dispatch)
            };

            if !consumed {
                // event was not consumed lower in the chain and is bubbling up to us.
                self.handle_event(event, state, dispatch_chain)
            } else {
                // consumed
                true
            }
        } else {
            self.handle_event(event, state, dispatch_chain)
        }
    }

    /// Add a flexbox layout style.
    pub fn apply_flex_style(&mut self, flex_style: &yoga::FlexStyle) {
        self.flexbox.apply_style(flex_style);
    }
}

/// Represents the user-accessible properties of an item in the hierarchy.
/// This is separated from the rest of the item data (children, behavior)
/// to allow multiple mutable borrows of different aspects of an item.
pub struct Item {
    /// The ID of the item. Unique among all nodes within an instance of `Ui`.
    pub id: ItemID,
    /// The calculated bounds of the item.
    pub layout: Layout,
    // Non-layout styles associated to this item.
    //pub style: Style,
    /// Cached calculated styles.
    pub style: ComputedStyle,
    /// CSS classes.
    pub css_classes: Vec<String>,
    /// Additional inline CSS properties.
    pub inline_styles: Vec<css::PropertyDeclaration>
}

impl Item {
    pub fn new(id: ItemID) -> Item {
        Item {
            id,
            layout: Layout::default(),
            style: ComputedStyle::default(),
            css_classes: Vec::new(),
            inline_styles: Vec::new()
        }
    }

    pub fn add_class(&mut self, class: &str) {
        self.css_classes.push(class.to_owned());
    }

    /*pub fn with_measure<F: Fn(&mut Item, &Renderer) -> ContentMeasurement + 'static>(
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
    }*/

    /*pub fn get_custom_data<D: Any>(&self) -> &D {
        self.custom_data.as_ref().unwrap().downcast_ref::<D>().unwrap()
    }

    pub fn get_custom_data_mut<D: Any>(&mut self) -> &mut D {
        self.custom_data.as_mut().unwrap().downcast_mut::<D>().unwrap()
    }*/

    /*pub fn apply_styles<'b, I>(&mut self, styles: I)
    where
        I: IntoIterator<Item = &'b yoga::FlexStyle>,
    {
        self.flexbox.apply_styles(styles);
    }*/
}
