use std::any::Any;
use super::vdom::*;
use super::input::*;
use glutin::WindowEvent;

pub trait Component: Any
{
    /*fn init() -> Self where Self: Sized + Default {
        Default::default()
    }*/

    /// One-time initialization.
    fn mount(&mut self, _elem: &RetainedNode)
    {
    }

    /// Called once per frame.
    fn post_frame(&mut self)
    {
    }

    /// Callback to handle an event passed to the item during the capturing phase.
    fn capture_event(&mut self,
                     _elem: &RetainedNode,
                     _event: &WindowEvent,
                     _input_state: &InputState) -> EventResult
    {
        EventResult::pass()
    }

    /// Callback to handle an event during bubbling phase.
    fn event(&mut self,
             _elem: &RetainedNode,
             _event: &WindowEvent,
             _input_state: &InputState) -> EventResult
    {
        EventResult::pass()
    }
}

pub trait ComponentAny: Component + Any
{
    fn as_mut_any(&mut self) -> &mut Any;
    fn as_mut_component(&mut self) -> &mut Component;
}

impl<T> ComponentAny for T where T: Component
{
    fn as_mut_any(&mut self) -> &mut Any {
        self
    }

    fn as_mut_component(&mut self) -> &mut Component {
        self
    }
}
