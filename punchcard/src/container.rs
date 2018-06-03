//! Widget creation interface.
use super::behavior::{Behavior, BehaviorAny};
use super::item::{Item, ItemNode};
use super::{
    Color, ComputedStyle, ContentMeasurement, ElementState, InputState, ItemID, Layout, Renderer,
    UiState,
};
use glutin::{MouseButton, VirtualKeyCode, WindowEvent};
use indexmap::{
    map::{Entry, OccupiedEntry, VacantEntry}, IndexMap,
};
use yoga;
use yoga::prelude::*;

use std::cell::Cell;
use std::fmt::Display;

/// Helper trait for window events.
pub trait WindowEventExt {
    /// If item was clicked by the main mouse button.
    fn clicked(&self) -> bool;
}

impl WindowEventExt for WindowEvent {
    fn clicked(&self) -> bool {
        match self {
            &WindowEvent::MouseInput { state, button, .. }
                if state == ElementState::Released && button == MouseButton::Left =>
            {
                true
            }
            _ => false,
        }
    }
}

/// A helper type for the construction of item hierarchies.
/// Creation or modification of new child items go though instances of UiContainer.
pub struct UiContainer<'a> {
    /// Reference to the global UI state.
    ui_state: &'a mut UiState,
    /// The ID of the item (parent of the children).
    pub id: ItemID,
    /// Children of the item.
    children: &'a mut IndexMap<ItemID, ItemNode>,
    /// Flexbox node.
    flexbox: &'a mut yoga::Node,
    /// Current number of children in item.
    cur_index: usize,
}

impl<'a> UiContainer<'a> {
    /// Gets the child item with the specified item ID, or create a new child with this ID if it
    /// doesn't exist.
    /// Returns:
    /// - a UIContainer for the newly created child item
    /// - a mutable reference to the item properties `&'mut Item`
    /// - a mutable reference to the `Behavior`
    pub(super) fn new_item<'b, F>(
        &'b mut self,
        new_item_id: ItemID,
        f: F,
    ) -> (UiContainer<'b>, &'b mut Item, &'b mut BehaviorAny)
    where
        F: FnOnce(ItemID) -> ItemNode,
    {
        // when inserting a child item:
        //      - if index matches: OK
        //      - else: swap the item at the correct location, mark it for relayout
        //              - insert item at the end, swap_remove item at index, put item back
        let cur_index = self.cur_index;
        let item_reinsert = {
            let entry = self.children.entry(new_item_id);
            // TODO accelerate common case (no change) by looking by index first
            match entry {
                Entry::Vacant(ref entry) => {
                    // entry is vacant: must insert at current location
                    Some(f(new_item_id))
                }
                Entry::Occupied(mut entry) => {
                    let index = entry.index();
                    // if the child item exists, see if its index corresponds
                    if cur_index != index {
                        // item has moved: extract the item from its previous position
                        self.flexbox.remove_child(&mut entry.get_mut().flexbox);
                        Some(entry.remove())
                    } else {
                        // child item exists and has not moved
                        None
                    }
                }
            }
            // drop borrow by entry()
        };

        if let Some(mut item) = item_reinsert {
            // must insert or reinsert an item at the correct index
            // insert last
            self.flexbox
                .insert_child(&mut item.flexbox, cur_index as u32);
            let len = self.children.len();
            self.children.insert(new_item_id, item);
            if cur_index != len {
                // we did not insert it at the correct position: need to swap the newly inserted element in place
                // remove element at target position: last inserted item now in position
                let kv = self.children.swap_remove_index(cur_index).unwrap();
                // reinsert previous item
                self.children.insert(kv.0, kv.1);
            //debug!("item {:016X} moved to {}", new_item_id, cur_index);
            } else {
                //debug!("item {:016X} inserted at {}", new_item_id, cur_index);
            }
        } else {
            //debug!("item {} at {} did not move", new_item_id, cur_index);
        }

        let item_node = self.children.get_index_mut(cur_index).unwrap().1;
        self.cur_index += 1;

        // clear inline CSS styles (they are respecified each frame)
        //item_node.item.inline_styles.clear();

        // 'deconstruct' the node into non-aliasing mutable borrows of its components.
        // This prevents headaches with the borrow checker down the line.
        (
            UiContainer {
                ui_state: self.ui_state,
                children: &mut item_node.children,
                flexbox: &mut item_node.flexbox,
                id: new_item_id,
                cur_index: 0,
            },
            &mut item_node.item,
            item_node.behavior.as_mut(),
        )
    }

    /// Create the UiContainer for the root item.
    pub(super) fn new_root(
        id: ItemID,
        item_node: &'a mut ItemNode,
        ui_state: &'a mut UiState,
    ) -> UiContainer<'a> {
        use std::any::Any;

        UiContainer {
            ui_state,
            children: &mut item_node.children,
            flexbox: &mut item_node.flexbox,
            id,
            cur_index: 0,
        }
    }

    pub(super) fn finish(mut self) {
        // TODO useless?
        // remove all extra children
        let num_children = self.children.len();
        if self.cur_index != num_children {
            debug!("removing {} extra children", num_children - self.cur_index);
            //self.flexbox.
        }
        for i in self.cur_index..num_children {
            self.children.swap_remove_index(i);
        }
    }

    /// TODO document.
    fn item_or_popup<S, B, F>(&mut self, id: S, class: &str, init: B, is_popup: bool, f: F)
    where
        S: Into<String>,
        B: Behavior,
        F: FnOnce(&mut UiContainer, &mut Item, &mut B),
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.ui_state.id_stack.push_id(&id_str);

        {
            use std::any::Any;
            let (mut ui, item, behavior) = self.new_item(id, move |id| {
                let mut node = ItemNode::new(id, Box::new(init));
                node.item.add_class(class);
                node
            });
            let behavior = behavior
                .as_mut_any()
                .downcast_mut()
                .expect("downcast to behavior type failed");
            f(&mut ui, item, behavior);
            ui.finish()
        }

        if is_popup {
            // add to popup list
            let popup_path = self.ui_state.id_stack.0.clone();
            self.ui_state.popups.push(popup_path);
        }

        self.ui_state.id_stack.pop_id();
    }

    /// TODO document.
    pub fn item<S, B, F>(&mut self, id: S, class: &str, init: B, f: F)
    where
        S: Into<String>,
        B: Behavior,
        F: FnOnce(&mut UiContainer, &mut Item, &mut B),
    {
        self.item_or_popup(id, class, init, false, f)
    }

    /// TODO document.
    pub fn popup<S, B, F>(&mut self, id: S, class: &str, init: B, f: F)
    where
        S: Into<String>,
        B: Behavior,
        F: FnOnce(&mut UiContainer, &mut Item, &mut B),
    {
        self.item_or_popup(id, class, init, true, f)
    }
}
