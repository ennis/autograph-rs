//! Layout panels (vbox and hbox), collapsible panels and floating panels.
use prelude::*;

///
/// Dummy.
/// Just a dummy element with a fixed size for testing.
///
#[derive(Default)]
struct Dummy;

/*impl Component for Dummy
{
    fn event(&mut self, elem: &RetainedNode, event: &WindowEvent, input_state: &InputState) -> EventResult {
        //debug!("dummy({:016X}): event received={:?}", elem.id, event);
        EventResult::pass()
    }
}*/

pub fn dummy(dom: &mut DomSink, size: (u32,u32))
{
    //dom.component("dummy", Dummy, |state,dom| {
    dom.div("dummy", |_|{}).set_size((size.0 as f32, size.1 as f32));
    //});
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
