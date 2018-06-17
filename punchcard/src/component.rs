use std::any::Any;
use super::vdom::*;
use super::input::*;
use glutin::WindowEvent;

pub trait Component: Any + Default
{
    /// One-time initialization.
    fn mount(&mut self, _elem: &RetainedElement)
    {
    }

    /// Callback to handle an event passed to the item during the capturing phase.
    fn capture_event(&mut self,
                     _elem: &RetainedElement,
                     _event: &WindowEvent,
                     _input_state: &mut InputState) -> bool
    {
        false
    }

    /// Callback to handle an event during bubbling phase.
    fn event(&mut self,
             _elem: &RetainedElement,
             _event: &WindowEvent,
             _input_state: &mut InputState) -> bool
    {
        false
    }

}