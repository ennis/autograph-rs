use super::behavior::{BehaviorAny};
use super::input::DispatchChain;
use super::input::InputState;
use super::layout::{ContentMeasurement, Layout};
use super::renderer::{DrawList, Renderer};
use super::style::{CachedStyle};
use super::{ItemID, Ui};

use indexmap::{
    map::Entry, IndexMap,
};
use glutin::{WindowEvent};
use yoga;

/// Data structure (indexmap) containing the children of a node.
pub(super) struct ItemChildren(pub(super) IndexMap<ItemID,ItemNode>);

impl ItemChildren
{
    pub(super) fn new() -> ItemChildren
    {
        ItemChildren(IndexMap::new())
    }

    /// Returns true if the item has moved.
    pub(super) fn create_or_update<NewFn, MoveFn>(&mut self, id: ItemID, index: usize, new_fn: NewFn, move_fn: MoveFn) -> &mut ItemNode
    where
        NewFn: FnOnce(ItemID) -> ItemNode,
        MoveFn: FnOnce(&mut ItemNode, bool),
    {
        let (moving, node_reinsert) = {
            let entry = self.0.entry(id);
            // TODO accelerate common case (no change) by looking by index first
            match entry {
                Entry::Vacant(_) => {
                    // entry is vacant: must insert at current location
                    (false, Some(new_fn(id)))
                }
                Entry::Occupied(mut entry) => {
                    let child_index = entry.index();
                    // if the child item exists, see if its index corresponds
                    if index != child_index {
                        // item has moved: extract the item from its previous position
                        (true, Some(entry.remove()))
                    } else {
                        // child item exists and has not moved
                        (false, None)
                    }
                }
            }
            // drop borrow by entry()
        };

        // reinsert item if necessary
        if let Some(mut node) = node_reinsert {
            // moving node
            move_fn(&mut node, moving);
            let len = self.0.len();
            self.0.insert(id, node);
            if index != len {
                // we did not insert it at the correct position:
                // need to swap the newly inserted element in place
                // remove element at target position: last inserted item now in position
                let kv = self.0.swap_remove_index(index).unwrap();
                // reinsert previous item
                self.0.insert(kv.0, kv.1);
            }
        }

        let node = self.0.get_index_mut(index).unwrap().1;
        node
    }

    /// Cleanup items from previous generation.
    pub(super) fn truncate(&mut self, trunc_size: usize) {
        let size = self.0.len();
        if trunc_size != size {
            debug!("removing {} extra children", size - trunc_size);
        }
        for i in trunc_size..size {
            self.0.swap_remove_index(i);
        }
    }
}


/// Represents a node in the item hierarchy.
pub(super) struct ItemNode {
    /// The corresponding node in the flexbox layout hierarchy.
    pub(super) flexbox: yoga::Node,
    /// Child nodes of this item.
    pub(super) children: ItemChildren,
    /// A set of callbacks describing the behavior of the item during deferred processing.
    /// Each widget has its own implementation of ItemBehavior that also stores
    /// internal state specific to the widget.
    /// See `Behavior` for more information.
    pub(super) behavior: Box<BehaviorAny>,
    /// User-facing properties of the item.
    pub(super) item: Item,
    // Last update generation.
    //pub(super) generation: usize
}

impl ItemNode {
    pub(super) fn new(id: ItemID, behavior: Box<BehaviorAny>) -> ItemNode {
        let mut n = ItemNode {
            children: ItemChildren::new(),
            flexbox: yoga::Node::new(),
            behavior,
            item: Item::new(id),
        };
        n.behavior.init(&mut n.item);
        n
    }

    pub(super) fn measure(&mut self, renderer: &Renderer) -> ContentMeasurement {
        self.behavior.measure(&mut self.item, renderer)
    }

    pub(super) fn draw(&mut self, draw_list: &mut DrawList) {
        self.behavior.draw(&mut self.item, draw_list)
    }

    pub(super) fn hit_test(&self, pos: (f32, f32)) -> bool {
        self.item.layout.is_point_inside(pos)
    }

    fn handle_event(
        &mut self,
        event: &WindowEvent,
        ui: &mut Ui,
        dispatch_chain: DispatchChain,
    ) -> bool {
        let capturing = ui.is_item_capturing(dispatch_chain.current_id());
        let mut input_state = InputState {
            ui,
            dispatch_chain,
            capturing,
            focused: false,
        };
        self.behavior.event(&mut self.item, event, &mut input_state)
    }

    fn capture_event(
        &mut self,
        event: &WindowEvent,
        ui: &mut Ui,
        dispatch_chain: DispatchChain,
    ) -> bool {
        let capturing = ui.is_item_capturing(dispatch_chain.current_id());
        let mut input_state = InputState {
            ui,
            dispatch_chain,
            capturing,
            focused: false,
        };
        self.behavior
            .capture_event(&mut self.item, event, &mut input_state)
    }

    /// Insert a new child node.
    pub(super) fn get_or_insert_child<NewFn>(&mut self, id: ItemID, index: usize, new_fn: NewFn) -> &mut ItemNode
    where
        NewFn: FnOnce(ItemID) -> ItemNode,
    {
        let flexbox = &mut self.flexbox;
        self.children.create_or_update(id, index, new_fn, |node, moving| {
            if moving {
                flexbox.remove_child(&mut node.flexbox);
            }
            flexbox.insert_child(&mut node.flexbox, index as u32);
        })
    }

    /// Propagates an event through this item to its children.
    pub(super) fn propagate_event(
        &mut self,
        event: &WindowEvent,
        ui: &mut Ui,
        dispatch_chain: DispatchChain,
    ) -> bool {
        // capture stage.
        if self.capture_event(event, ui, dispatch_chain) {
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
                    .children.0
                    .get_mut(&next_dispatch.current_id())
                    .expect("item deleted");
                next.propagate_event(event, ui, next_dispatch)
            };

            if !consumed {
                // event was not consumed lower in the chain and is bubbling up to us.
                self.handle_event(event, ui, dispatch_chain)
            } else {
                // consumed
                true
            }
        } else {
            self.handle_event(event, ui, dispatch_chain)
        }
    }

    /*
    /// Add a flexbox layout style.
    pub fn apply_flex_style(&mut self, flex_style: &yoga::FlexStyle) {
        self.flexbox.apply_style(flex_style);
    }*/
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
    pub(super) css_class: Option<String>,
    /// Dynamic layout overrides.
    /// TODO: handle this outside/after flexbox layout to avoid costly calculations.
    pub(super) layout_overrides: LayoutOverrides,
    /// Popup z-order. None if not a popup.
    pub(super) z_order: Option<i32>,
}

impl Item {
    pub fn new(id: ItemID) -> Item {
        Item::new_popup(id, None)
    }


    pub fn new_popup(id: ItemID, z_order: Option<i32>) -> Item {
        Item {
            id,
            layout: Layout::default(),
            style: CachedStyle::default(),
            styles_dirty: true,
            css_class: None,
            layout_overrides: LayoutOverrides::default(),
            z_order,
        }
    }

    pub fn is_popup(&self) -> bool {
        self.z_order.is_some()
    }

    pub fn bring_to_front(&mut self) {
        if self.z_order.is_some() {
            // XXX this is random
            self.z_order = Some(1000);
        }
    }

    pub fn set_class(&mut self, class: &str) {
        self.css_class = Some(class.to_owned());
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
