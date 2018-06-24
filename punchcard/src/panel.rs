//! Layout panels (vbox and hbox), collapsible panels and floating panels.
use super::*;
use super::behavior::*;
use super::input::*;
use yoga::prelude::*;

///
/// Dummy.
/// Just a dummy element with a fixed size for testing.
///
pub fn dummy(dom: &mut DomSink, size: (u32,u32))
{
    let node = dom.div("dummy", |_|{});
    node.layout_overrides.width = Some((size.0 as f32).point());
    node.layout_overrides.height = Some((size.1 as f32).point());
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
             _elem: &RetainedNode,
             _event: &WindowEvent,
             _input_state: &InputState) -> EventResult
    {
        EventResult::pass()
    }
}

pub fn collapsing_panel(dom: &mut DomSink, title: impl Into<String>, f: impl FnOnce(&mut DomSink))
{
    let title = title.into();
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

/// Adds children to the dom if condition is true. Equivalent to:
/// ```
/// if condition {
///     f(dom);
/// }
/// ```
/// This is a convenience function to be used within the dom!() macro,
/// which does not handle if-statements.
pub fn condition(dom: &mut DomSink, condition: bool, f: impl FnOnce(&mut DomSink)) {
    if condition {
        f(dom);
    }
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
