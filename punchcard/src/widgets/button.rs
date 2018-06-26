//! Buttons
use prelude::*;

///
/// Button.
///
#[derive(Default)]
struct Button
{
}

impl Component for Button
{
    fn event(&mut self, _elem: &RetainedNode, event: &WindowEvent, input_state: &InputState) -> EventResult {
        EventResult::pass()
    }
}

pub fn button(dom: &mut DomSink, size: (u32,u32))
{
    dom.component::<Button,_,_,_>("button", |_|{}, |state,children,dom| {
        let node = dom.div("button", |_|{});
    });
}

