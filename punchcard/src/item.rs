use super::behavior::{Behavior, BehaviorAny};
use super::container::WindowEventExt;
use super::css;
use super::input::DispatchChain;
use super::input::InputState;
use super::layout::{ContentMeasurement, Layout};
use super::renderer::{DrawItem, DrawItemKind, DrawList, Renderer};
use super::style::{CachedStyle, ComputedStyle};
use super::ResourceStore;
use super::{ItemID, UiState};
use indexmap::{map::Entry, map::OccupiedEntry, map::VacantEntry, IndexMap};

use glutin::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, WindowEvent};
use yoga;

use std::any::Any;
use std::cell::Cell;
use std::mem;

/// Represents a node in the item hierarchy.
pub(super) struct ItemNode {
    /// The corresponding node in the flexbox layout hierarchy.
    pub(super) flexbox: yoga::Node,
    /// Child nodes of this item.
    pub(super) children: IndexMap<ItemID, ItemNode>,
    /// A set of callbacks describing the behavior of the item during deferred processing.
    /// Each widget has its own implementation of ItemBehavior that also stores
    /// internal state specific to the widget.
    /// See `Behavior` for more information.
    pub(super) behavior: Box<BehaviorAny>,
    /// User-facing properties of the item.
    pub(super) item: Item,
}

impl ItemNode {
    pub fn new(id: ItemID, behavior: Box<BehaviorAny>) -> ItemNode {
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

    pub fn draw(&mut self, draw_list: &mut DrawList) {
        self.behavior.draw(&mut self.item, draw_list)
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
        self.behavior
            .capture_event(&mut self.item, event, &mut input_state)
    }

    /// Propagates an event through this item to its children.
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
                let next = self
                    .children
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct LayoutOverrides {
    pub left: Option<yoga::StyleUnit>,
    pub right: Option<yoga::StyleUnit>,
    pub top: Option<yoga::StyleUnit>,
    pub bottom: Option<yoga::StyleUnit>,
    pub width: Option<yoga::StyleUnit>,
    pub height: Option<yoga::StyleUnit>,
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
    pub style: CachedStyle,
    /// Whether the CSS classes have changed since last style calculation.
    pub(super) styles_dirty: bool,
    /// CSS classes.
    pub(super) css_classes: Vec<String>,
    /// Dynamic layout overrides.
    /// TODO: handle this outside/after flexbox layout to avoid costly calculations.
    pub(super) layout_overrides: LayoutOverrides,
    /// Popup z-order. None if not a popup.
    pub(super) z_order: Option<u32>,
}

impl Item {
    pub fn new(id: ItemID) -> Item {
        Item::new_popup(id, None)
    }

    pub fn new_popup(id: ItemID, z_order: Option<u32>) -> Item {
        Item {
            id,
            layout: Layout::default(),
            style: CachedStyle::default(),
            styles_dirty: true,
            css_classes: Vec::new(),
            layout_overrides: LayoutOverrides::default(),
            z_order,
        }
    }

    pub fn is_popup(&self) -> bool {
        self.z_order.is_some()
    }

    pub fn add_class(&mut self, class: &str) {
        self.css_classes.push(class.to_owned());
        self.styles_dirty = true;
    }

    /// Overrides the position of the widget.
    /// Pass None to let the layout decide.
    pub fn set_position(&mut self, x: Option<yoga::StyleUnit>, y: Option<yoga::StyleUnit>) {
        self.layout_overrides.left = x;
        self.layout_overrides.top = y;
    }

    /// Overrides the measured width & height of the widget.
    pub fn set_size(&mut self, width: Option<yoga::StyleUnit>, height: Option<yoga::StyleUnit>) {
        self.layout_overrides.width = width;
        self.layout_overrides.height = height;
    }
}
