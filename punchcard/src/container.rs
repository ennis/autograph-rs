//! Widget creation interface.
use super::behavior::{Behavior, BehaviorAny};
use super::item::{Item, ItemNode, ItemChildren};
use super::{
    ElementState, ItemID, Ui,
};
use glutin::{MouseButton, WindowEvent};
use indexmap::{
    map::{Entry}, IndexMap,
};
use yoga;

/// Helper trait for window events.
pub trait WindowEventExt {
    /// If item was clicked by the main mouse button.
    fn clicked(&self) -> bool;
    fn mouse_down(&self) -> bool;
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

    fn mouse_down(&self) -> bool {
        match self {
            &WindowEvent::MouseInput { state, button, .. }
            if state == ElementState::Pressed && button == MouseButton::Left =>
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
    pub(super) ui: &'a mut Ui,
    /// The ID of the item (parent of the children).
    pub id: ItemID,
    /// Children of the item.
    pub(super) children: &'a mut ItemChildren,
    /// Flexbox node.
    pub(super) flexbox: &'a mut yoga::Node,
    /// Current number of children in item.
    pub(super) cur_index: usize,
}

// Ui -> (UiContainer + Item)
// Children + Flexbox + ID -> UiContainer

impl<'a> UiContainer<'a> {

    /// Brings a popup to the front, on top of everything else.
    /*pub fn bring_to_front(&mut self, item: &mut Item) {

    }*/

    pub(super) fn finish(self) {
        // remove all extra children
        self.children.truncate(self.cur_index);
    }

    /// TODO document.
    pub fn item<S, B, F>(&mut self, id: S, class: &str, init: B, f: F)
    where
        S: Into<String>,
        B: Behavior,
        F: FnOnce(&mut UiContainer, &mut Item, &mut B),
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.ui.id_stack.push_id(&id_str);

        {
            // insert new item
            let index = self.cur_index;
            let flexbox = &mut self.flexbox;
            let node = self.children.create_or_update(id, index, move |id| {
                let mut node = ItemNode::new(id, Box::new(init));
                node.item.set_class(class);
                node
            }, |node, moving| {
                if moving {
                    flexbox.remove_child(&mut node.flexbox);
                }
                flexbox.insert_child(&mut node.flexbox, index as u32);
            });
            self.cur_index += 1;

            // split into mutable borrows of the contents to avoid headaches
            let behavior = &mut node.behavior;
            let item = &mut node.item;
            let mut container = UiContainer {
                ui: self.ui,
                children: &mut node.children,
                flexbox: &mut node.flexbox,
                id,
                cur_index: 0,
            };
            let behavior = behavior
                .as_mut_any()
                .downcast_mut()
                .expect("downcast to behavior type failed");
            f(&mut container, item, behavior);
            behavior.post_frame(item, container.ui.frame_index);
            container.finish()
        }

        self.ui.id_stack.pop_id();
    }


    pub fn popup<S, B, F>(&mut self, id: S, class: &str, init: B, f: F)
        where
            S: Into<String>,
            B: Behavior,
            F: FnOnce(&mut UiContainer, &mut Item, &mut B),
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.ui.id_stack.push_id(&id_str);

        // we don't care about the insertion order for popups
        // (they don't influence the layout of child items)
        {
            let frontmost_z = self.ui.roots.0.len() as i32;
            // we extract the item so that we own it no strings attached.
            let mut node = self.ui.roots.0.remove(&id).unwrap_or_else(|| {
                let mut node = ItemNode::new(id, Box::new(init));
                node.item.set_class(class);
                node.item.z_order = Some(frontmost_z);
                node
            });

            {
                let mut container = UiContainer {
                    ui: self.ui,
                    children: &mut node.children,
                    flexbox: &mut node.flexbox,
                    id,
                    cur_index: 0,
                };
                let item = &mut node.item;
                let behavior = node.behavior.as_mut()
                    .as_mut_any()
                    .downcast_mut()
                    .expect("downcast to behavior type failed");
                f(&mut container, item, behavior);
                container.finish();
            }

            // insert node into popup list
            self.ui.roots.0.insert(id, node);
        }

        self.ui.id_stack.pop_id();
    }
}

// new popup: z_order = topmost_z; topmost_z += 1;
// bring to front: same