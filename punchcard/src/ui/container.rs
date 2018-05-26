use super::item::{ItemBehaviorAny, ItemNode};
use super::{Color, ContentMeasurement, ElementState, InputState, Item,
            ItemBehavior, ItemID, Layout, Renderer, ComputedStyle, UiState, VirtualKeyCode, WindowEvent};
use indexmap::{map::{Entry, OccupiedEntry, VacantEntry},
               IndexMap};
use yoga;
use yoga::prelude::*;
use glutin::MouseButton;

use std::fmt::Display;
use std::cell::Cell;

/// Helper trait for window events.
pub trait WindowEventExt
{
    /// If item was clicked by the main mouse button.
    fn clicked(&self) -> bool;
}

impl WindowEventExt for WindowEvent
{
    fn clicked(&self) -> bool {
        match self {
            &WindowEvent::MouseInput {
                state,
                button,
                ..
            } if state == ElementState::Released && button == MouseButton::Left => true,
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
    /// - a mutable reference to the `ItemBehavior`
    pub(super) fn new_item<'b, F>(
        &'b mut self,
        new_item_id: ItemID,
        f: F
    ) -> (UiContainer<'b>, &'b mut Item, &'b mut ItemBehaviorAny)
    where F: FnOnce(ItemID) -> ItemNode
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
    pub fn item<S, Behavior, F>(&mut self, id: S, class: &str, init: Behavior, f: F)
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


pub trait Interpolable: Copy+'static
{
    fn lerp(a: Self, b: Self, t: f32) -> Self;
    fn ratio(a: Self, b: Self, t: Self) -> f32;
}

impl Interpolable for f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        //assert!(b > a);
        t * (b - a) + a
    }

    fn ratio(a: f32, b: f32, t: f32) -> f32 {
        (t - a) / (b - a)
    }
}

impl Interpolable for i32 {
    fn lerp(a: i32, b: i32, t: f32) -> i32 {
        //assert!(b > a);
        (t * (b - a) as f32 + a as f32).round() as i32
    }

    fn ratio(a: i32, b: i32, t: i32) -> f32 {
        (t - a) as f32 / (b - a) as f32
    }
}

impl Interpolable for u32 {
    fn lerp(a: u32, b: u32, t: f32) -> u32 {
        //assert!(b > a);
        (t as f64 * (b - a) as f64 + a as f64).round() as u32
    }

    fn ratio(a: u32, b: u32, t: u32) -> f32 {
        (t - a) as f32 / (b - a) as f32
    }
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
        impl ItemBehavior for VBox {
        }

        self.item(id, "vbox", VBox, |ui, item, _| {
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

        self.item(id, "hbox", HBox, |ui, item, _| {
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
        self.item(id, "scroll", ScrollState { pos: 0.0 }, |mut ui, item, scroll| {
            let top = -scroll.pos;
            /*style!(ui.flexbox,
                FlexDirection(yoga::FlexDirection::Column),
                FlexGrow(1.0),
                Margin(4.0 pt),
                Top(top.point())
            );*/

            f(ui);
        });
    }

    ///
    /// Text with class.
    ///
    pub fn text_class<S>(&mut self, text: S, class: &str) -> ItemResult
        where
            S: Into<String> + Clone,
    {
        //=====================================
        // behavior
        struct Text(String);
        impl ItemBehavior for Text {
            fn draw(&mut self, item: &mut Item, renderer: &mut Renderer) {
                renderer.draw_text(&self.0, &item.layout, &item.style);
            }

            fn measure(&mut self, item: &mut Item, renderer: &Renderer) -> ContentMeasurement {
                let m = renderer.measure_text(&self.0, &item.style);
                ContentMeasurement {
                    width: Some(m),
                    height: Some(item.style.font.font_size),
                }
            }
        }

        //=====================================
        // hierarchy
        self.item(text.clone(), class, Text(text.into()), |_, _, _| {});

        //=====================================
        // result
        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Text.
    ///
    pub fn text<S>(&mut self, text: S) -> ItemResult
    where
        S: Into<String> + Clone,
    {
        self.text_class(text, "text")
    }

    ///
    /// Button.
    ///
    pub fn button<S>(&mut self, label: S)
        where
            S: Into<String>
    {
        let label = label.into();
        struct Button;
        impl ItemBehavior for Button {}
        self.item(label.clone(), "button", Button, |ui,_,_| {
            ui.text_class(label, "button-label");
        });
    }


    ///
    /// Slider with a f32 backing value.
    ///
    pub fn slider<S, T: Interpolable+Display>(&mut self, label: S, value: &mut T, min: T, max: T)
    where
        S: Into<String>,
    {
        use num::clamp;

        //=====================================
        // slider
        struct Slider<T: Interpolable> {
            value: T,
            min: T,
            max: T,
            dirty: bool
        };

        impl<T: Interpolable> Slider<T> {
            fn sync(&mut self, value: &mut T) {
                if self.dirty {
                    *value = self.value;
                } else {
                    self.value = *value;
                }
                self.dirty = false;
            }

            fn set_ratio(&mut self, ratio: f32) {
                let value = <T as Interpolable>::lerp(self.min, self.max, ratio);
                self.set_value(value);
            }

            fn set_value(&mut self, value: T) {
                self.value = value;
                self.dirty = true;
            }

            fn ratio(&self) -> f32 {
                <T as Interpolable>::ratio(self.min, self.max, self.value)
            }
        }

        impl<T: Interpolable+Display> ItemBehavior for Slider<T> {
            fn event(
                &mut self,
                item: &mut Item,
                event: &WindowEvent,
                input_state: &mut InputState,
            ) -> bool {
                // update the slider current value from the current cursor position
                let mut update_slider_pos = |layout: &Layout, cursor_pos: (f32, f32)| {
                    let (cx, _) = cursor_pos;
                    self.set_ratio(clamp((cx - layout.left) / layout.width(),0.0,1.0));
                    debug!("slider pos={}", self.value);
                };

                // debug!("Slider capture {:016X} {:?}", itemid, event);
                match event {
                    // clicked inside the slider layout rect
                    &WindowEvent::MouseInput {
                        state: elem_state, ..
                    }  =>
                    {
                        if elem_state == ElementState::Pressed {
                            // capture events
                            input_state.set_capture();
                            update_slider_pos(&item.layout, input_state.cursor_pos());
                        }
                        //debug!("slider event");
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
        self.item(label, "slider", Slider { value: *value, min, max, dirty: false }, |ui, item, slider| {
            use std::mem::swap;
            slider.min = min;
            slider.max = max;
            slider.sync(value);
            let knob_pos = slider.ratio();

            ui.item("bar", "slider-bar", Bar, |ui, item, _| {
                ui.item("knob", "slider-knob", Knob, |ui, item, _| {
                    item.set_position(Some((100.0*knob_pos).percent()), None);
                });
            });
        });
    }

    ///
    /// Collapsing header.
    ///
    pub fn collapsing_panel<S,F>(&mut self, id: S, f: F)
        where
            S: Into<String>,
            F: FnOnce(&mut UiContainer),
    {
        let label = id.into();

        //=====================================
        // bar
        struct CollapsingPanel {
            collapsed: bool,
        }
        impl ItemBehavior for CollapsingPanel {
            fn event(&mut self, item: &mut Item, event: &WindowEvent, input_state: &mut InputState) -> bool {
                debug!("panel event");
                if event.clicked() {
                    self.collapsed = !self.collapsed;
                    true
                }
                else { false }
            }
        }

       /* struct CollapsingPanelHeader;
        impl ItemBehavior for CollapsingPanelHeader
        {
            fn event(&mut self, item: &mut Item, event: &WindowEvent, input_state: &mut InputState) -> bool {
                // transparent to events
                false
            }
        }*/

        self.item(label.clone(), "collapsing-panel", (), |ui, item, state| {
            let mut collapsed = false;
            ui.item("header", "collapsing-panel-header", CollapsingPanel { collapsed: true }, |ui, item, state| {
                ui.text(label.clone());
                collapsed = state.collapsed;
            });

            if !collapsed {
                ui.item("contents", "collapsing-panel-contents", (), |ui, item, _| {
                    f(ui);
                });
            }
        });
    }

}

