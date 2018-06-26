//! Layout panels (vbox and hbox), collapsible panels and floating panels.
use prelude::*;

///
/// Dummy.
/// Just a dummy element with a fixed size for testing.
///
#[derive(Default)]
struct Dummy
{
}

impl Component for Dummy
{
    fn event(&mut self, _elem: &RetainedNode, event: &WindowEvent, input_state: &InputState) -> EventResult {
        debug!("dummy: event received={:?}; cursor pos={:?}", event, input_state.cursor_pos);
        EventResult::pass()
    }
}

pub fn dummy(dom: &mut DomSink, size: (u32,u32))
{
    dom.component::<Dummy,_,_,_>("dummy", |_|{}, |state,children,dom| {
        let node = dom.div("dummy", |_|{});
        node.layout_overrides.width = Some((size.0 as f32).point());
        node.layout_overrides.height = Some((size.1 as f32).point());
    });
}

///
/// Vertical layout box.
///
pub fn vbox(dom: &mut DomSink, f: impl FnOnce(&mut DomSink))
{
    // don't really care about the ID here since it's stateless.
    dom.div("vbox", f);
}

///
/// Horizontal layout box.
///
pub fn hbox(dom: &mut DomSink, f: impl FnOnce(&mut DomSink))
{
    // don't really care about the ID here since it's stateless.
    dom.div("hbox", f);
}
