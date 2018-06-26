use prelude::*;

///
/// Draggable panel.
/// Very similar to collapsing panels.
///
pub struct FloatingPanel {
    /// Is the panel collapsed (only title bar shown).
    collapsed: bool,
    /// Position drag.
    drag: DragBehavior,
}

impl Default for FloatingPanel {
    fn default() -> Self {
        FloatingPanel {
            collapsed: false,
            drag: DragBehavior::default(),
        }
    }
}

impl Component for FloatingPanel
{
    fn event(&mut self,
             elem: &RetainedNode,
             event: &WindowEvent,
             input_state: &InputState) -> EventResult
    {
        /*if event.mouse_down() {
            // TODO this should be done in render()
            //elem.bring_to_front();
        }*/
        self.drag.event(elem, event, input_state)
        // TODO collapsing behavior
    }
}

pub fn floating_panel(dom: &mut DomSink, title: impl Into<String>, f: impl FnOnce(&mut DomSink))
{
    let title = title.into();
    dom.component::<FloatingPanel,_,_,_>(title.clone(), f, |state,children,dom| {
        dom.div("floating", |dom| {
            dom.div("floating-header", |dom| {
                dom.text(title);
            });
            if !state.collapsed {
                dom.div("floating-contents", |dom| {
                    dom.push(children);
                });
            }
        });
    });
}
