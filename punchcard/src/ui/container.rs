use super::item::{ItemBehaviorAny, ItemNode};
use super::{Color, ContentMeasurement, DummyBehavior, ElementState, InputState, Item,
            ItemBehavior, ItemID, Layout, Renderer, Style, UiState, VirtualKeyCode, WindowEvent};
use indexmap::{map::{Entry, OccupiedEntry, VacantEntry},
               IndexMap};
use yoga;
use yoga::prelude::*;

use std::cell::Cell;

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

// Styling/layout:
// - align_item(f32)
// - left(f32)
// - right(f32)
// - top(f32)
// - bottom(f32)
//

/// Styling - layout
/*impl<'a, State: 'static> UiContainer<'a, State> {
    pub fn align_item(&mut self, align: yoga::Align) {
        self.item.flexbox.set_align_items(align);
    }
    pub fn align_content(&mut self, align: yoga::Align) {
        self.item.flexbox.set_align_content(align);
    }
    pub fn align_self(&mut self, align: yoga::Align) {
        self.item.flexbox.set_align_self(align);
    }
}*/

/// Styling - rendering
/*impl<'a, State: 'static> UiContainer<'a, State> {
    pub fn border_color(&mut self, rgba: Color) {
        self.item.style.border_color = Some(rgba);
    }

    pub fn background_color(&mut self, rgba: Color) {
        self.item.style.background_color = Some(rgba);
    }
}*/

impl<'a> UiContainer<'a> {
    /// Gets the child item with the specified item ID, or create a new child with this ID if it
    /// doesn't exist.
    /// Returns:
    /// - a UIContainer for the newly created child item
    /// - a mutable reference to the item properties `&'mut Item`
    /// - a mutable reference to the `ItemBehavior`
    pub(super) fn new_item<'b>(
        &'b mut self,
        new_item_id: ItemID,
        init_behavior: Box<ItemBehaviorAny>,
    ) -> (UiContainer<'b>, &'b mut Item, &'b mut ItemBehaviorAny) {
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
                    Some(ItemNode::new(new_item_id, init_behavior))
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
    }

    /// TODO document.
    pub fn item<S, Behavior, F>(&mut self, id: S, init: Behavior, f: F)
    where
        S: Into<String>,
        Behavior: ItemBehavior,
        F: FnOnce(&mut UiContainer, &mut Item, &mut Behavior),
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.ui_state.id_stack.push_id(&id_str);

        {
            use std::any::Any;
            let (mut ui, item, behavior) = self.new_item(id, Box::new(init));
            let behavior = behavior
                .as_mut_any()
                .downcast_mut()
                .expect("downcast to behavior type failed");
            f(&mut ui, item, behavior);
            //ui.finish()
        }

        self.ui_state.id_stack.pop_id();
    }
}

pub struct ScrollState {
    pub scroll_pos: f32,
}

struct SliderState {
    pos: f32,
}

pub struct ItemResult {
    /// The item was clicked since the last call
    pub clicked: bool,
    /// The mouse is hovering over the item
    pub hover: bool,
}

impl<'a> UiContainer<'a> {
    ///
    /// Vertical layout box.
    ///
    pub fn vbox<S, F>(&mut self, id: S, f: F) -> ItemResult
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        struct VBox;
        impl ItemBehavior for VBox {}

        self.item(id, VBox, |mut ui, item, _| {
            style!(ui.flexbox,
                FlexDirection(yoga::FlexDirection::Column),
                Margin(2.0 pt)
            );
            f(ui);
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Horizontal layout box.
    ///
    pub fn hbox<S, F>(&mut self, id: S, f: F) -> ItemResult
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        struct HBox;
        impl ItemBehavior for HBox {}

        self.item(id, HBox, |mut ui, item, _| {
            style!(ui.flexbox,
                FlexDirection(yoga::FlexDirection::Row),
                Margin(2.0 pt)
            );
            f(ui);
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Scrollable panel.
    ///
    pub fn scroll<S, F>(&mut self, id: S, f: F)
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        //=====================================
        // behavior
        struct ScrollState {
            pub pos: f32,
        };
        impl ItemBehavior for ScrollState {
            fn event(
                &mut self,
                item: &mut Item,
                event: &WindowEvent,
                input_state: &mut InputState,
            ) -> bool {
                match event {
                    &WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(VirtualKeyCode::Up) => {
                            debug!("Scroll up");
                            self.pos -= 10.0;
                        }
                        Some(VirtualKeyCode::Down) => {
                            debug!("Scroll down");
                            self.pos += 10.0;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                // always capture?
                false
            }
        }

        //=====================================
        // hierarchy
        self.item(id, ScrollState { pos: 0.0 }, |mut ui, item, scroll| {
            let top = -scroll.pos;
            style!(ui.flexbox,
                FlexDirection(yoga::FlexDirection::Column),
                FlexGrow(1.0),
                Margin(4.0 pt),
                Top(top.point())
            );

            f(ui);
        });
    }

    ///
    /// Text.
    ///
    pub fn text<S>(&mut self, text: S) -> ItemResult
    where
        S: Into<String> + Clone,
    {
        //=====================================
        // behavior
        struct Text(String);
        impl ItemBehavior for Text {
            fn draw(&mut self, item: &mut Item, renderer: &mut Renderer) {
                renderer.draw_text(&self.0, &item.layout, &item.calculated_style);
            }

            fn measure(&mut self, item: &mut Item, renderer: &Renderer) -> ContentMeasurement {
                let m = renderer.measure_text(&self.0, &item.calculated_style);
                ContentMeasurement {
                    width: Some(m),
                    height: item.calculated_style.font_size,
                }
            }
        }

        //=====================================
        // hierarchy
        self.item(text.clone(), Text(text.into()), |_, _, _| {});

        //=====================================
        // result
        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Slider with a f32 backing value.
    ///
    /// Unresolved issue: synchronization of the value with the internal state?
    ///
    /// if nothing happened: state <- value.
    ///
    /// if state has changed: state -> value.
    ///
    pub fn slider_f32<S>(&mut self, label: S, value: &mut f32, min: f32, max: f32)
    where
        S: Into<String>,
    {
        use num::clamp;

        //=====================================
        // slider
        struct Slider {
            pos: f32,
            min: f32,
            max: f32,
        };
        impl ItemBehavior for Slider {
            fn event(
                &mut self,
                item: &mut Item,
                event: &WindowEvent,
                input_state: &mut InputState,
            ) -> bool {
                // update the slider current value from the current cursor position
                let mut update_slider_pos = |layout: &Layout, cursor_pos: (f32, f32)| {
                    let (cx, _) = cursor_pos;
                    let off = (cx - layout.left) / layout.width() * (self.max - self.min);
                    let off = clamp(off, self.min, self.max);
                    self.pos = off;
                    debug!("slider pos={}", self.pos);
                };

                // debug!("Slider capture {:016X} {:?}", itemid, event);
                match event {
                    // clicked inside the slider layout rect
                    &WindowEvent::MouseInput {
                        state: elem_state, ..
                    } if elem_state == ElementState::Pressed =>
                    {
                        // capture events
                        input_state.set_capture();
                        update_slider_pos(&item.layout, input_state.cursor_pos());
                        true
                    }
                    &WindowEvent::CursorMoved { position, .. } => {
                        if input_state.capturing {
                            update_slider_pos(&item.layout, input_state.cursor_pos());
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
        }

        //=====================================
        // knob
        struct Knob;
        impl ItemBehavior for Knob {
        }

        //=====================================
        // bar
        struct Bar;
        impl ItemBehavior for Bar {
        }

        //=====================================
        // hierarchy
        self.item(label, Slider { pos: *value, min, max }, |ui, item, slider| {
            use std::mem::swap;
            slider.min = min;
            slider.max = max;
            //swap(value, &mut slider.pos);
            let knob_pos = slider.pos;

            style!(
                ui.flexbox,
                FlexDirection(yoga::FlexDirection::Column),
                JustifyContent(yoga::Justify::Center),
                AlignItems(yoga::Align::Stretch),
                Height(20.0 pt)
            );

            ui.item("bar", Bar, |ui, item, _| {
                item.style.set_background_color((0.3, 0.3, 0.3, 1.0));
                item.style.set_border_radius(2.0);

                style!(
                    ui.flexbox,
                    FlexDirection(yoga::FlexDirection::Row),
                    AlignItems(yoga::Align::Center),
                    Height(5.0 pt)
                );

                // the knob
                ui.item("knob", Knob, |ui, item, _| {
                    item.style.set_background_color((0.0, 1.0, 0.0, 1.0));
                    item.style.set_border_radius(2.0);

                    style!(ui.flexbox,
                        FlexDirection(yoga::FlexDirection::Row),
                        Position(yoga::PositionType::Relative),
                        Width(10.0 pt),
                        Height(10.0 pt),
                        Left((100.0*knob_pos).percent()));
                });
            });
        });
    }
}
