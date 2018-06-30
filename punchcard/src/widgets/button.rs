//! Buttons
use prelude::*;

///
/// Button.
///
pub fn button(dom: &mut DomSink) -> bool
{
    dom.component("button", ButtonBehavior::default(), |state, dom| {
        let node = dom.div("button", |_|{});
        state.clicked()
    })
}

///
/// Alternative to button(), if you prefer mutable refs.
///
pub fn button_alt(dom: &mut DomSink, clicked: &mut bool)
{
    *clicked = button(dom);
}


