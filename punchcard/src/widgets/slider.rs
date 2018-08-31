//! Sliders.
use prelude::*;
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

struct Slider<T: Interpolable>
{
    value: T,
    min: T,
    max: T,
    dirty: bool,
}

impl<T: Interpolable> Slider<T> {
    fn new(value: T, min: T, max: T) -> Slider<T>
    {
        Slider {
            value,
            min,
            max,
            dirty: false,
        }
    }

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


impl<T: Interpolable + Display> Component for Slider<T> {
    fn event(&mut self,
             event: &WindowEvent,
             bounds: &Bounds,
             input_state: &InputState) -> EventResult
    {
        use num::clamp;

        // update the slider current value from the current cursor position
        let mut update_slider_pos = |cursor_pos: (f32, f32)| {
            let (cx, _) = cursor_pos;
            self.set_ratio(clamp((cx - bounds.left) / bounds.width(), 0.0, 1.0));
        };

        // debug!("Slider capture {:016X} {:?}", itemid, event);
        match event {
            // clicked inside the slider layout rect
            &WindowEvent::MouseInput {
                state, ..
            } => {
                if state == ElementState::Pressed {
                    // capture events
                    update_slider_pos(input_state.cursor_pos());
                    EventResult::stop().set_capture()
                }
                else {
                    //debug!("slider event");
                    EventResult::stop()
                }
            }
            &WindowEvent::CursorMoved { .. } => {
                if input_state.is_capturing() {
                    update_slider_pos(input_state.cursor_pos());
                    EventResult::stop().set_capture()
                } else {
                    EventResult::pass()
                }
            }
            _ => EventResult::pass(),
        }
    }
}


pub fn slider<S: Into<String>, T: Interpolable + Display>(dom: &mut DomSink, label: S, value: &mut T, min: T, max: T)
{
    let label = label.into();
    dom.component(label.clone(), Slider::new(*value, min, max), |state,dom| {
        state.min = min;
        state.max = max;
        state.sync(value);
        dom.div("slider", |dom| {
            dom.div("slider-bar", |dom| {
                dom.div("slider-knob", |dom| {}).set_x_percent(100.0*state.ratio());
            });
        });
    });
}


/*
struct AppState
{
    a: f32,
    b: f32,
    c: f32,
}

// This is a component!
// The first parameter must be some context struct.
// A render() function cannot return a visual tree because components
// are not bound yet.
// UiSink: collects visual items.
fn gui_for_app_state(ui: &mut UiSink, app: &mut AppState)
{
    ui! {ui,
        @Slider(value=&mut app.a, min=0.0, max=1.0) {

        };
        @Slider(value=&mut app.b, min=0.0, max=1.0) {

        };
        @Slider(value=&mut app.c, min=0.0, max=1.0) {

        };
        @Button(label="clear", on_click=|| {
            app.c = 0.0;
        })
    }
    //---------------------------------
    // Expands to:

    ui.add_component::<Slider>(/* props */ SliderProps::new {
        value: &mut app.a,
        min: 0.0,
        max: 1.0
    }, |_| {});    // drops the ref!
}
*/
