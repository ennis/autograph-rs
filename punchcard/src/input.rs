//! Input handling.
use super::{ElementID, Ui};
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
    /// The path (hierarchy of IDs) to the element that is capturing the mouse pointer.
    pub(super) id: ElementID,
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

/// Struct passed to event handlers.
pub struct InputState<'a> {
    /// TODO document
    pub(super) ui: &'a mut Ui,
    /// Dispatch chain that the event is travelling along.
    pub(super) dispatch_chain: DispatchChain<'a>,
    /// Whether the item received this event because it has been captured.
    /// TODO make this private, replace with method.
    pub capturing: bool,
    /// Whether the item received this event because it has focus.
    /// TODO make this private, replace with method.
    pub focused: bool,
}

impl<'a> InputState<'a> {
    /// Signals that the current item in the dispatch chain should capture all events.
    pub fn set_capture(&mut self) {
        // TODO
        unimplemented!()
        //self.ui.set_capture(self.dispatch_chain.current_chain().into());
        //self.capturing = true;
    }

    /// Signals that the current item should have focus.
    pub fn set_focus(&mut self) {
        // TODO
        unimplemented!()
        //self.ui.set_focus(self.dispatch_chain.current_chain().into());
    }

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

    /// Release the capture. This fails (silently) if the current item is not
    /// capturing events.
    pub fn release_capture(&mut self) {
        // TODO
        unimplemented!()
        // check that we are capturing
        /*if self.capturing {
            self.ui.release_capture()
        } else {
            warn!("trying to release capture without capturing");
        }*/
    }

    /// Get the current cursor position.
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.ui.cursor_pos
    }
}
