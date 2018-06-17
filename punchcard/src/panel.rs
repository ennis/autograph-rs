//! Layout panels (vbox and hbox), collapsible panels and floating panels.
use super::*;
use super::behavior::*;

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

///
/// Collapsing panel.
///
struct CollapsingHeader
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
             _elem: &RetainedElement,
             _event: &WindowEvent,
             _input_state: &mut InputState) -> bool
    {
        false
    }
}

pub fn collapsing_panel(dom: &mut DomSink, title: impl Into<String>, f: impl FnOnce(&mut DomSink))
{
    let mut title = title.into();
    dom.component::<CollapsingHeader,_,_,_>(title.clone(), f, |state,children,dom| {
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

///
/// Draggable panel.
/// Very similar to collapsing panels.
///
struct FloatingPanel {
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
             elem: &RetainedElement,
             event: &WindowEvent,
             input_state: &mut InputState) -> bool
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
    let mut title = title.into();
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
