use super::{Color, ContentMeasurement, ElementState, InputState, Item, ItemID, Layout, Renderer,
            Style, UiState, VirtualKeyCode, WindowEvent};
use indexmap::{map::{Entry, OccupiedEntry, VacantEntry},
               IndexMap};
use yoga;
use yoga::prelude::*;

use std::cell::Cell;

pub struct UiContainer<'a, State: 'static> {
    /// Reference to the global UI state.
    ui_state: &'a mut UiState,
    /// Wrapped item ID.
    pub id: ItemID,
    /// Reference to the wrapped item.
    pub item: &'a mut Item,
    /// Current number of children in item.
    cur_index: usize,
    /// The custom data. No allocation is made if T is ()
    item_state: Box<State>,
}

// Styling/layout:
// - align_item(f32)
// - left(f32)
// - right(f32)
// - top(f32)
// - bottom(f32)
//

/// Styling - layout
impl<'a, State: 'static> UiContainer<'a, State> {
    pub fn align_item(&mut self, align: yoga::Align) {
        self.item.flexbox.set_align_items(align);
    }
    pub fn align_content(&mut self, align: yoga::Align) {
        self.item.flexbox.set_align_content(align);
    }
    pub fn align_self(&mut self, align: yoga::Align) {
        self.item.flexbox.set_align_self(align);
    }
}

/// Styling - rendering
/*impl<'a, State: 'static> UiContainer<'a, State> {
    pub fn border_color(&mut self, rgba: Color) {
        self.item.style.border_color = Some(rgba);
    }

    pub fn background_color(&mut self, rgba: Color) {
        self.item.style.background_color = Some(rgba);
    }
}*/

impl<'a, State: 'static> UiContainer<'a, State> {
    pub(super) fn new_item<'s, U: 'static, F: FnOnce() -> Item>(
        &'s mut self,
        new_item_id: ItemID,
        f: F,
    ) -> UiContainer<'s, U> {
        // when inserting a child item:
        //      - if index matches: OK
        //      - else: swap the item at the correct location, mark it for relayout
        //              - insert item at the end, swap_remove item at index, put item back
        let cur_index = self.cur_index;
        let item_reinsert = {
            let entry = self.item.children.entry(new_item_id);
            // TODO accelerate common case (no change) by looking by index first
            match entry {
                Entry::Vacant(ref entry) => {
                    // entry is vacant: must insert at current location
                    Some(f())
                }
                Entry::Occupied(mut entry) => {
                    let index = entry.index();
                    // if the child item exists, see if its index corresponds
                    if cur_index != index {
                        // item has moved: extract the item from its previous position
                        self.item.flexbox.remove_child(&mut entry.get_mut().flexbox);
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
            self.item
                .flexbox
                .insert_child(&mut item.flexbox, cur_index as u32);
            let len = self.item.children.len();
            self.item.children.insert(new_item_id, item);
            if cur_index != len {
                // we did not insert it at the correct position: need to swap the newly inserted element in place
                // remove element at target position: last inserted item now in position
                let kv = self.item.children.swap_remove_index(cur_index).unwrap();
                // reinsert previous item
                self.item.children.insert(kv.0, kv.1);
            //debug!("item {:016X} moved to {}", new_item_id, cur_index);
            } else {
                //debug!("item {:016X} inserted at {}", new_item_id, cur_index);
            }
        } else {
            //debug!("item {} at {} did not move", new_item_id, cur_index);
        }

        let new_item = self.item.children.get_index_mut(cur_index).unwrap().1;
        let item_state = new_item.extract_state();
        self.cur_index += 1;

        UiContainer {
            ui_state: self.ui_state,
            item: new_item,
            id: new_item_id,
            cur_index: 0,
            item_state,
        }
    }

    pub(super) fn new_root(
        id: ItemID,
        item: &'a mut Item,
        ui_state: &'a mut UiState,
    ) -> UiContainer<'a, State> {
        let item_state = item.extract_state();
        UiContainer {
            ui_state,
            item,
            id,
            cur_index: 0,
            item_state,
        }
    }

    pub(super) fn finish(mut self) {
        let state = self.item_state;
        self.item.replace_state(state);
    }
}

impl<'a, State: 'static> UiContainer<'a, State> {
    /// Set the draw callback
    pub fn draw<F>(&mut self, f: F)
    where
        F: Fn(&mut Item, &mut State, &mut Renderer) + 'static,
    {
        self.item.draw = Some(Box::new(move |item, renderer| {
            item.with_extract_state(|item, state| f(item, state, renderer))
        }));
    }

    /// Set the measure callback, **if** it has not already been set.
    pub fn measure<F>(&mut self, f: F)
    where
        F: Fn(&mut Item, &mut State, &Renderer) -> ContentMeasurement + 'static,
    {
        self.item.with_measure(move |item, renderer| {
            item.with_extract_state(|item, state| f(item, state, renderer))
        });
    }

    ///
    pub fn input_event<F>(&mut self, f: F)
    where
        F: Fn(&mut Item, &mut State, &WindowEvent, &mut InputState) -> bool + 'static,
    {
        self.item.event_handler.get_or_insert_with(|| {
            Box::new(move |item, event, input_state| {
                item.with_extract_state(|item, state| f(item, state, event, input_state))
            })
        });
    }

    /// Returns a mutable reference to the style of the item.
    /// Shorthand for `self.item.style`.
    pub fn styles_mut(&mut self) -> &mut Style {
        &mut self.item.style
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

impl<'a, State> UiContainer<'a, State> {
    pub fn item<S, U, F>(&mut self, id: S, init: U, f: F)
    where
        S: Into<String>,
        U: 'static,
        F: FnOnce(&mut UiContainer<U>),
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.ui_state.id_stack.push_id(&id_str);

        {
            let mut ui = self.new_item(id, || {
                let mut item = Item::new();
                item.state = Cell::new(Some(Box::new(init)));
                item
            });
            f(&mut ui);
            ui.finish()
        }

        self.ui_state.id_stack.pop_id();
    }

    pub fn vbox<S, F>(&mut self, id: S, f: F) -> ItemResult
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer<()>),
    {
        self.item(id, (), |ui| {
            ui.draw(|item, state, renderer| {
                renderer.draw_rect(&item.layout, &item.style);
            });
            style!(ui.item,
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

    pub fn hbox<S, F>(&mut self, id: S, f: F) -> ItemResult
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer<()>),
    {
        self.item(id, (), |ui| {
            ui.draw(|item, state, renderer| {
                renderer.draw_rect(&item.layout, &item.style);
            });
            style!(ui.item,
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

    /// a scrollable panel
    pub fn scroll<S, F>(&mut self, id: S, f: F)
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer<ScrollState>),
    {
        self.item(id, ScrollState { scroll_pos: 0.0 }, |ui| {
            ui.input_event(|item, state, event, input_state| {
                match event {
                    &WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(VirtualKeyCode::Up) => {
                            debug!("Scroll up");
                            state.scroll_pos -= 10.0;
                        }
                        Some(VirtualKeyCode::Down) => {
                            debug!("Scroll down");
                            state.scroll_pos += 10.0;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                // always capture?
                false
            });

            let top = -ui.item_state.scroll_pos;

            style!(ui.item,
                FlexDirection(yoga::FlexDirection::Column),
                FlexGrow(1.0),
                Margin(4.0 pt),
                Top(top.point())
            );

            ui.draw(|item, state, renderer| {
                renderer.draw_rect(&item.layout, &item.style);
            });

            f(ui);
        });
    }

    pub fn text<S>(&mut self, text: S) -> ItemResult
    where
        S: Into<String> + Clone,
    {
        self.item(text.clone(), text.into(), |ui| {
            ui.draw(|item, state, renderer| {
                renderer.draw_text(state, &item.layout, &item.style);
            });

            ui.measure(|item, state, renderer| {
                let m = renderer.measure_text(state.as_ref(), &item.style);
                ContentMeasurement {
                    width: Some(m),
                    height: item.style.font_size,
                }
            });
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    pub fn slider_f32<S>(&mut self, label: S, value: &mut f32, min: f32, max: f32)
    where
        S: Into<String>,
    {
        use num::clamp;

        self.item(label, SliderState { pos: 0.0 }, |ui| {
            // DRAW should be immediate!
            // issue: don't know the layout yet...
            // could specify a different closure each time, but it will reallocate!
            ui.draw(|item, state, renderer| {
                // draw the slider bar
                // draw the knob
                renderer.draw_rect(&item.layout, &item.calculated_style);
            });

            let itemid = ui.id;
            let knob_pos = ui.item_state.pos;
            // debug!("knob pos={}", knob_pos);

            ui.input_event(move |item, state, event, input_state| {
                // update the slider current value from the current cursor position
                let update_slider_pos =
                    |state: &mut SliderState, layout: &Layout, cursor_pos: (f32, f32)| {
                        let (cx, _) = cursor_pos;
                        let off = (cx - layout.left) / layout.width() * (max - min);
                        let off = clamp(off, min, max);
                        state.pos = off;
                        debug!("slider pos={}", state.pos);
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
                        update_slider_pos(state, &item.layout, input_state.cursor_pos());
                        true
                    }
                    &WindowEvent::CursorMoved { position, .. } => {
                        if input_state.capturing {
                            update_slider_pos(state, &item.layout, input_state.cursor_pos());
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            });

            style!(
                ui.item,
                FlexDirection(yoga::FlexDirection::Column),
                JustifyContent(yoga::Justify::Center),
                AlignItems(yoga::Align::Stretch),
                Height(20.0 pt)
            );

            // the bar
            ui.item("bar", (), |bar| {
                bar.draw(|item, state, renderer| {
                    renderer.draw_rect(&item.layout, &item.calculated_style);
                });

                bar.item.style.set_background_color((0.3, 0.3, 0.3, 1.0));
                bar.item.style.set_border_radius(2.0);

                style!(
                    bar.item,
                    FlexDirection(yoga::FlexDirection::Row),
                    AlignItems(yoga::Align::Center),
                    Height(5.0 pt)
                );

                // the knob
                bar.item("knob", (), |knob| {
                    knob.draw(|item, state, renderer| {
                        renderer.draw_rect(&item.layout, &item.calculated_style);
                    });

                    knob.item.style.set_background_color((0.0, 1.0, 0.0, 1.0));
                    knob.item.style.set_border_radius(2.0);

                    style!(knob.item,
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
