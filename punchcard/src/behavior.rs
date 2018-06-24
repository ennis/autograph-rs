//! Common item behaviors.
use super::input::{WindowEventExt,InputState,EventResult};
use super::vdom::*;
use super::layout::ContentMeasurement;
use glutin::{ElementState, WindowEvent};

pub struct CheckboxBehavior {
    pub checked: bool,
}

impl Default for CheckboxBehavior {
    fn default() -> CheckboxBehavior {
        CheckboxBehavior {
            checked: false
        }
    }
}

impl CheckboxBehavior {
    pub fn event(
        &mut self,
        _elem: &mut RetainedNode,
        event: &WindowEvent,
        _input_state: &mut InputState,
    ) -> EventResult {
        if event.clicked() {
            self.checked = !self.checked;
        }
        EventResult::stop()
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

impl Default for DragBehavior {
    fn default() -> DragBehavior {
        DragBehavior {
            drag: None,
            start_value: None,
            drag_started: true,
        }
    }
}

impl DragBehavior {

    pub fn handle_drag(&mut self, value: &mut (f32, f32)) -> bool
    {
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

    pub fn event(
        &mut self,
        _elem: &RetainedNode,
        event: &WindowEvent,
        input_state: &InputState,
    ) -> EventResult {
        //debug!("EVENT {:016X}", item.id);
        // drag behavior:
        // - on mouse button down: capture, set click pos
        // - on cursor move: update offset
        let mut should_capture_inputs = false;
        let captured = match event {
            &WindowEvent::MouseInput { state, .. } => {
                if state == ElementState::Pressed {
                    // capture events
                    should_capture_inputs = true;
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
                if input_state.is_capturing() {
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

        if !input_state.is_capturing() {
            // drag end
            self.drag = None;
            self.start_value = None;
        }

        if captured {
            EventResult::stop().set_capture()
        } else {
            EventResult::pass()
        }
    }
}
