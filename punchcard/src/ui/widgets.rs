use super::container::{UiContainer,WindowEventExt};
use super::{Color, ContentMeasurement, ElementState, InputState, Item,
            ItemBehavior, ItemID, Layout, Renderer, ComputedStyle, UiState, VirtualKeyCode, WindowEvent};
use super::item::DragBehavior;
use std::fmt::Display;
use yoga::prelude::*;

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

        struct CollapsingPanel {
            collapsed: bool,
        }
        impl ItemBehavior for CollapsingPanel {
            fn event(&mut self, item: &mut Item, event: &WindowEvent, input_state: &mut InputState) -> bool {
                if event.clicked() {
                    self.collapsed = !self.collapsed;
                    true
                }
                    else { false }
            }
        }

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

    ///
    /// Draggable panel.
    /// Very similar to collapsing panels.
    ///
    pub fn floating_panel<S,F>(&mut self, id: S, f: F)
        where
            S: Into<String>,
            F: FnOnce(&mut UiContainer)
    {
        let label = id.into();

        //============================================
        // Panel
        struct FloatingPanel {
            /// Is the panel collapsed (only title bar shown).
            collapsed: bool,
            /// Position drag.
            drag: DragBehavior,
        }

        impl FloatingPanel
        {
            fn new() -> Self {
                FloatingPanel {
                    collapsed: false,
                    drag: DragBehavior::new(),
                }
            }
        }

        impl ItemBehavior for FloatingPanel {
            fn event(&mut self, item: &mut Item, event: &WindowEvent, input_state: &mut InputState) -> bool {
                self.drag.event(item, event, input_state)
                // TODO collapsing behavior
            }
        }

        self.popup(label.clone(), "floating-panel", FloatingPanel::new(), |ui, panel_item, panel_behavior| {
            let mut position = (panel_item.layout.left, panel_item.layout.top);
            if panel_behavior.drag.handle_drag(&mut position) {
                panel_item.set_position(Some(position.0.point()), Some(position.1.point()));
            }

            ui.item("header", "floating-panel-header", (), |ui, item, _| {
                ui.text(label.clone());
            });

            ui.item("contents", "floating-panel-contents", (), |ui, item, _| {
                if !panel_behavior.collapsed {
                    f(ui);
                }
            });
            ui.item("resize-handle", "floating-panel-resize-handle", DragBehavior::new(), |ui, handle_item, handle_behavior| {
                let mut size = (panel_item.layout.width(), panel_item.layout.height());
                if handle_behavior.handle_drag(&mut size) {
                    //debug!("DRAG SIZE {}x{}", size.0, size.1);
                    panel_item.set_size(Some(size.0.point()), Some(size.1.point()));
                }
            });
        });
    }
}
