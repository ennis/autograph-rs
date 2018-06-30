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
    /// Position
    position: (f32, f32),
    size: (f32, f32)
}

impl Default for FloatingPanel {
    fn default() -> Self {
        FloatingPanel {
            collapsed: false,
            drag: DragBehavior::default(),
            position: (0.0, 0.0),
            size: (120.0, 120.0)
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
    dom.aggregate_component(title.clone(), FloatingPanel::default(), f, |state,children,dom| {
        state.drag.handle_drag(&mut state.position);
        dom.div("floating", |dom| {
            dom.div("floating-header", |dom| {
                dom.text(title);
            });
            if !state.collapsed {
                dom.div("floating-contents", |dom| {
                    dom.push(children);
                });
            }
            draggable_handle(dom, "floating-resize-handle", &mut state.size);
        }).set_position(state.position).set_size(state.size);
        //debug!("position={},{} size={}x{}", state.position.0, state.position.1, state.size.0, state.size.1);
    });
}

#[derive(Default)]
struct DraggableHandle
{
    drag: DragBehavior
}

impl Component for DraggableHandle
{
    fn event(&mut self, elem: &RetainedNode, event: &WindowEvent, input_state: &InputState) -> EventResult {
        self.drag.event(elem, event, input_state)
    }
}

pub fn draggable_handle(dom: &mut DomSink, class: &str, value: &mut (f32,f32))
{
    dom.component("handle", DraggableHandle::default(), |state,dom| {
        state.drag.handle_drag(value);
        dom.div(class, |_|{});
    });
}