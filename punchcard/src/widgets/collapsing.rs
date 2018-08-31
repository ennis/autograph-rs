use prelude::*;

///
/// Collapsing panel.
///
pub struct CollapsingHeader
{
    checkbox_behavior: CheckboxBehavior,
}

impl Default for CollapsingHeader
{
    fn default() -> CollapsingHeader {
        CollapsingHeader {
            checkbox_behavior: CheckboxBehavior::default()
        }
    }
}

impl Component for CollapsingHeader
{
    /// Callback to handle an event during bubbling phase.
    fn event(&mut self,
             _event: &WindowEvent,
             _bounds: &Bounds,
             _input_state: &InputState) -> EventResult
    {
        EventResult::pass()
    }
}

pub fn collapsing_panel(dom: &mut DomSink, title: impl Into<String>, f: impl FnOnce(&mut DomSink))
{
    let title = title.into();
    dom.aggregate_component(title.clone(), CollapsingHeader::default(), f, |state,children,dom| {
        dom.div("collapsing", |dom| {
            dom.div("collapsing-header", |dom| {
                dom.text(title);
            });
            if !state.checkbox_behavior.checked {
                dom.div("collapsing-contents", |dom| {
                    dom.push(children);
                });
            }
        });
    });
}
