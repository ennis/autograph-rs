//! Common item behaviors.
use super::container::WindowEventExt;
use super::input::InputState;
use super::item::Item;
use super::layout::ContentMeasurement;
use super::renderer::{DrawList, Renderer};
use glutin::{ElementState, WindowEvent};
use std::any::Any;

/// A set of callbacks that describes the behavior of an item for all deferred processing:
/// i.e., processing that happens outside the scope of the function calls that create or
/// update the item (the _immediate path_).
/// Typically, implementors of this trait are also used to store persistent internal state inside
/// items.
pub trait Behavior: Any {
    /// One-time initialization.
    fn init(&mut self, _item: &mut Item) {}

    /// once-per-frame initialization.
    fn post_frame(&mut self, _item: &mut Item, frame_index: usize) {}

    /// Draw the item to the specified renderer.
    fn draw(&mut self, item: &mut Item, draw_list: &mut DrawList) {
        draw_list.add_rect(item.layout.clone(), item.style.clone());
    }

    /// Measure the given item using the specified renderer.
    fn measure(&mut self, _item: &mut Item, _renderer: &Renderer) -> ContentMeasurement {
        ContentMeasurement {
            width: None,
            height: None,
        }
    }

    /// Callback to handle an event passed to the item during the capturing phase.
    fn capture_event(
        &mut self,
        _item: &mut Item,
        _event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        false
    }

    /// Callback to handle an event during bubbling phase.
    fn event(
        &mut self,
        _item: &mut Item,
        _event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        false
    }
}

/// A wrapper around ItemBehavior and Any traits. See:
/// https://github.com/rust-lang/rfcs/issues/2035
/// https://stackoverflow.com/questions/26983355
pub(super) trait BehaviorAny: Behavior + Any {
    fn as_mut_behavior(&mut self) -> &mut Behavior;
    fn as_mut_any(&mut self) -> &mut Any;
}

impl<T> BehaviorAny for T
where
    T: Behavior + Any,
{
    fn as_mut_behavior(&mut self) -> &mut Behavior {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
    }
}

//#[derive(Copy, Clone, Debug)]
//pub struct DummyBehavior;

impl Behavior for () {
    //fn draw(&mut self, _item: &mut Item, _renderer: &mut Renderer) {}

    fn measure(&mut self, _item: &mut Item, _renderer: &Renderer) -> ContentMeasurement {
        ContentMeasurement {
            width: None,
            height: None,
        }
    }

    fn capture_event(
        &mut self,
        _item: &mut Item,
        _event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        false
    }

    /// Callback to handle an event during bubbling phase.
    fn event(
        &mut self,
        _item: &mut Item,
        _event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        false
    }
}

struct Invisible;

impl Behavior for Invisible {
    fn draw(&mut self, _item: &mut Item, _draw_list: &mut DrawList) {}

    fn measure(&mut self, _item: &mut Item, _renderer: &Renderer) -> ContentMeasurement {
        ContentMeasurement {
            width: None,
            height: None,
        }
    }

    fn capture_event(
        &mut self,
        _item: &mut Item,
        _event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        // capture nothing
        false
    }

    fn event(
        &mut self,
        _item: &mut Item,
        _event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        // always bubble
        false
    }
}

/// InputBehavior: feed events, get info.
/*pub struct InputBehavior
{
    pub clicked: bool,
    pub drag: Option<DragState>
}*/

pub struct CheckboxBehavior {
    pub checked: bool,
}

impl CheckboxBehavior {
    pub fn new() -> CheckboxBehavior {
        CheckboxBehavior { checked: false }
    }
}

impl Behavior for CheckboxBehavior {


    fn event(
        &mut self,
        _item: &mut Item,
        event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> bool {
        if event.clicked() {
            self.checked = !self.checked;
        }
        true
    }
}

pub struct ButtonBehavior
{
    /// Whether the button has been clicked on the given frame.
    pub clicked: bool
}

impl ButtonBehavior
{
    pub fn new() -> ButtonBehavior {
        ButtonBehavior { clicked: false }
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }
}

impl Behavior for ButtonBehavior
{
    fn post_frame(&mut self, _item: &mut Item, _frame_index: usize) {
        // reset clicked flag.
        self.clicked = false;
    }

    fn event(&mut self, _item: &mut Item, event: &WindowEvent, input_state: &mut InputState) -> bool {
        if event.clicked() {
            self.clicked = true;
        }
        true
    }
}

pub struct DragState {
    /// Where the mouse pointer was when the dragging started.
    pub origin: (f32, f32),
    /// Current drag offset.
    pub offset: (f32, f32),
}

/// Common input behavior.
pub struct DragBehavior {
    pub drag: Option<DragState>,
    pub drag_started: bool,
    pub start_value: Option<(f32, f32)>,
}

impl DragBehavior {
    pub fn new() -> DragBehavior {
        DragBehavior {
            drag: None,
            start_value: None,
            drag_started: true,
        }
    }

    pub fn handle_drag(&mut self, value: &mut (f32, f32)) -> bool {
        if self.drag_started {
            self.start_value = Some(*value);
            self.drag_started = false;
        }

        if let Some(ref drag) = self.drag {
            if let Some(ref start) = self.start_value {
                *value = (start.0 + drag.offset.0, start.1 + drag.offset.1);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Behavior for DragBehavior {
    fn event(
        &mut self,
        _item: &mut Item,
        event: &WindowEvent,
        input_state: &mut InputState,
    ) -> bool {
        //debug!("EVENT {:016X}", item.id);
        // drag behavior:
        // - on mouse button down: capture, set click pos
        // - on cursor move: update offset
        let captured = match event {
            &WindowEvent::MouseInput { state, .. } => {
                if state == ElementState::Pressed {
                    // capture events
                    input_state.set_capture();
                    // starting drag
                    self.drag_started = true;
                    self.drag = Some(DragState {
                        origin: input_state.cursor_pos(),
                        offset: (0.0, 0.0),
                    });
                }
                true
            }
            &WindowEvent::CursorMoved { .. } => {
                if input_state.capturing {
                    let cursor_pos = input_state.cursor_pos();
                    if let Some(ref mut drag) = self.drag {
                        // continue drag, update offset
                        drag.offset = (cursor_pos.0 - drag.origin.0, cursor_pos.1 - drag.origin.1);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        if !input_state.capturing {
            // drag end
            self.drag = None;
            self.start_value = None;
        }
        captured
    }
}
