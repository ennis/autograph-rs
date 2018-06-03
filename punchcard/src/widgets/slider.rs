//! Sliders.
use super::super::*;
use std::fmt::Display;

pub trait Interpolable: Copy + 'static {
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
    /// Slider with a f32 backing value.
    ///
    pub fn slider<S, T: Interpolable + Display>(&mut self, label: S, value: &mut T, min: T, max: T)
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
            dirty: bool,
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

        impl<T: Interpolable + Display> Behavior for Slider<T> {
            fn event(
                &mut self,
                item: &mut Item,
                event: &WindowEvent,
                input_state: &mut InputState,
            ) -> bool {
                // update the slider current value from the current cursor position
                let mut update_slider_pos = |layout: &Layout, cursor_pos: (f32, f32)| {
                    let (cx, _) = cursor_pos;
                    self.set_ratio(clamp((cx - layout.left) / layout.width(), 0.0, 1.0));
                    debug!("slider pos={}", self.value);
                };

                // debug!("Slider capture {:016X} {:?}", itemid, event);
                match event {
                    // clicked inside the slider layout rect
                    &WindowEvent::MouseInput {
                        state: elem_state, ..
                    } => {
                        if elem_state == ElementState::Pressed {
                            // capture events
                            input_state.set_capture();
                            update_slider_pos(&item.layout, input_state.cursor_pos());
                        }
                        //debug!("slider event");
                        true
                    }
                    &WindowEvent::CursorMoved { .. } => {
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
        impl Behavior for Knob {}

        //=====================================
        // bar
        struct Bar;
        impl Behavior for Bar {}

        //=====================================
        // hierarchy
        self.item(
            label,
            "slider",
            Slider {
                value: *value,
                min,
                max,
                dirty: false,
            },
            |ui, _, slider| {
                slider.min = min;
                slider.max = max;
                slider.sync(value);
                let knob_pos = slider.ratio();

                ui.item("bar", "slider-bar", Bar, |ui, _, _| {
                    ui.item("knob", "slider-knob", Knob, |_, item, _| {
                        item.set_position(Some((100.0 * knob_pos).percent()), None);
                    });
                });
            },
        );
    }
}
