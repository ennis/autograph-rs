//! Input handling.
use super::{ElementID, Ui};
use super::id_tree::{Arena, NodeId};
use super::vdom::RetainedNode;
use glutin::{ElementState, WindowEvent, MouseButton};

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

/// Struct containing information about a pointer capture.
#[derive(Clone)]
pub(super) struct PointerCapture {
    /// Where the mouse button was at capture.
    pub(super) origin: (f32, f32),
    /// The path (hierarchy of IDs) to the visual node that is capturing the mouse pointer.
    pub(super) id: NodeId,
}

/// Describes the nature of the target of a dispatch chain.
#[derive(Copy, Clone, Debug)]
pub(super) enum DispatchTarget {
    /// The dispatch chain targets a captured item.
    Capture,
    /// The dispatch chain targets a focused item.
    Focus,
    /// The dispatch chain targets a leaf item that passed the cursor hit-test.
    HitTest,
}

/// Represents a dispatch chain: a chain of items that should receive an event.
#[derive(Copy, Clone)]
pub(super) struct DispatchChain<'a> {
    /// The items in the chain.
    pub(super) elements: &'a [ElementID],
    /// Current position in the chain.
    pub(super) current: usize,
    /// Reason for dispatch.
    pub(super) target: DispatchTarget,
}

impl<'a> DispatchChain<'a> {
    /// advance position in the chain
    pub(super) fn next(&self) -> Option<DispatchChain<'a>> {
        if self.current + 1 < self.elements.len() {
            Some(DispatchChain {
                elements: self.elements,
                current: self.current + 1,
                target: self.target,
            })
        } else {
            None
        }
    }

    /// Get the current item ID
    pub(super) fn current_id(&self) -> ElementID {
        self.elements[self.current]
    }

    /*
    /// Returns the final target of this dispatch chain.
    pub(super) fn target_id(&self) -> ItemID {
        *self.items.last().unwrap()
    }*/

    /// Returns the currently processed chain, including the current element.
    pub(super) fn current_chain(&self) -> &'a [ElementID] {
        &self.elements[0..=self.current]
    }
}

/// What should be done after passing an event.
pub enum EventPropagation
{
    /// Pass along to other handlers in the chain.
    Pass,
    /// Stop propagation.
    Stop,
    /// Stop propagation and capture all subsequent events.
    StopAndCapture,
}

pub struct EventResult
{
    pub(super) stop_propagation: bool,
    pub(super) set_capture: bool,
    pub(super) set_focus: bool
}

impl Default for EventResult {
    fn default() -> Self {
        EventResult {
            stop_propagation: false,
            set_capture: false,
            set_focus: false
        }
    }
}

impl EventResult {
    pub fn set_capture(self) -> Self {
        EventResult {
            set_capture: true,
            .. self
        }
    }

    pub fn set_focus(self) -> Self {
        EventResult {
            set_focus: true,
            .. self
        }
    }

    pub fn pass() -> Self {
        Default::default()
    }

    pub fn stop() -> Self {
        EventResult {
            stop_propagation: true,
            .. Default::default()
        }
    }
}

/// Struct passed to event handlers.
pub struct InputState {
    /// TODO document
    pub(super) cursor_pos: (f32,f32),
    pub(super) capture: Option<PointerCapture>,
    /// Whether the item received this event because it has focus.
    /// TODO make this private, replace with method.
    pub focused: bool,
    /// The frame index for the event.
    pub frame_index: usize,

}

impl InputState {
    /// Get the pointer capture origin position.
    pub fn get_capture_origin(&self) -> Option<(f32, f32)> {
        // TODO
        unimplemented!()
        //self.ui.capture.as_ref().map(|params| params.origin)
    }

    /// Get drag delta from start of capture.
    pub fn get_capture_drag_delta(&self) -> Option<(f32, f32)> {
        // TODO
        unimplemented!()
        /*self.ui.capture.as_ref().map(|params| {
            let (ox, oy) = params.origin;
            let (cx, cy) = self.ui.cursor_pos;
            (cx - ox, cy - oy)
        })*/
    }

    pub fn is_capturing(&self) -> bool {
        self.capture.is_some()
    }

    /// Get the current cursor position.
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.cursor_pos
    }
}
