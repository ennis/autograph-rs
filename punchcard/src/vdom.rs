use super::*;
use super::style::{apply_to_flex_node, Styles};

use std::fmt::Debug;
use std::rc::Rc;

/// Bits of DOM.
#[derive(Debug)]
pub enum Contents<T>
{
    Div(Vec<T>),
    Text(String)
}

/// CSS styles that can be set dynamically.
/// Everything else must be set through the CSS class.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct LayoutOverrides {
    pub left: Option<yoga::StyleUnit>,
    pub right: Option<yoga::StyleUnit>,
    pub top: Option<yoga::StyleUnit>,
    pub bottom: Option<yoga::StyleUnit>,
    pub width: Option<yoga::StyleUnit>,
    pub height: Option<yoga::StyleUnit>,
}

/// Data shared between the actual DOM and the VDOM.
#[derive(Debug)]
pub struct Element<T: Debug>
{
    pub(super) id: ElementID,
    pub(super) class: String,
    pub(super) contents: Contents<Element<T>>,
    pub(super) layout_overrides: LayoutOverrides,
    pub(super) extra: T
}

pub type VirtualElement = Element<()>;
pub type RetainedElement = Element<RetainedData>;

/// Actual DOM node (persistent across frames).
#[derive(Debug)]
pub struct RetainedData
{
    /// Cached calculated layout.
    pub(super) layout: Layout,
    /// Styles must be recalculated.
    pub(super) styles_dirty: bool,
    /// Resolved styles.
    pub(super) styles: Option<Rc<Styles>>,
    /// yoga Flexbox node
    pub(super) flex: yoga::Node,
}

impl VirtualElement
{
    pub fn new_text<S: Into<String>>(id: ElementID, class: &str, text: S) -> VirtualElement
    {
        VirtualElement {
            id,
            class: class.into(),
            layout_overrides: Default::default(),
            contents: Contents::Text(text.into()),
            extra: ()
        }
    }

    pub fn new_div(id: ElementID, class: &str, children: Vec<VirtualElement>) -> VirtualElement
    {
        VirtualElement {
            id,
            class: class.into(),
            layout_overrides: Default::default(),
            contents: Contents::Div(children),
            extra: ()
        }
    }

    pub(super) fn into_retained(self) -> RetainedElement
    {
        let mut flex = yoga::Node::new();

        let contents = match self.contents {
            Contents::Text(s) => Contents::Text(s),
            Contents::Div(mut children) => {
                let mut child_index = 0;
                Contents::Div(children.drain(..).map(|c| {
                    let mut c = c.into_retained();
                    flex.insert_child(&mut c.extra.flex, child_index);
                    child_index += 1;
                    c
                }).collect())
            }
        };

        RetainedElement {
            id: self.id,
            class: self.class,
            contents,
            layout_overrides: self.layout_overrides,
            extra: RetainedData {
                layout: Layout::default(),
                styles: None,
                styles_dirty: true,
                flex
            }
        }
    }

}

impl RetainedElement
{
    /// Update in place from a VDOM element.
    pub(super) fn update(&mut self, vdom: VirtualElement) {
        let data = &mut self.extra;

        // TODO compare classes and trigger restyle if necessary.
        self.class = vdom.class;

        // update the contents
        match vdom.contents {
            Contents::Div(mut vchildren) => {
                if let Contents::Div(ref mut children) = self.contents {
                    update_element_list(data, children, vchildren);
                } else {
                    self.contents = {
                        let mut children = Vec::new();
                        update_element_list(data, &mut children, vchildren);
                        Contents::Div(children)
                    };
                }
            },
            Contents::Text(text) => {
                self.contents = Contents::Text(text);
            }
        }
    }
}

fn update_element_list(parent: &mut RetainedData, retained: &mut Vec<RetainedElement>, mut vdom: Vec<VirtualElement>)
{
    //debug!("update_element_list retained={:#?}, vdom={:#?}", retained, vdom);
    let num_elem = vdom.len();
    'outer: for (vi,v) in vdom.drain(..).enumerate() {
        // the first vi elements are already updated.
        for ri in vi..retained.len() {
            if retained[ri].id == v.id {
                // matching node, update in place
                retained[ri].update(v);
                // swap flex nodes
                parent.flex.remove_child(&mut retained[ri].extra.flex);
                parent.flex.insert_child(&mut retained[ri].extra.flex, vi as u32);
                // swap at the correct position
                retained.swap(ri, vi);
                continue 'outer;
            }
        }
        // no matching element found in retained graph: insert a new one
        let mut new = v.into_retained();
        parent.flex.insert_child(&mut new.extra.flex, vi as u32);
        retained.push(new);
        // swap in position
        let last_index = retained.len()-1;
        retained.swap(last_index, vi);
    }
    // trim all extra elements
    retained.truncate(num_elem);
}


pub struct DomSink<'a>
{
    /// Ref to the root UI object.
    ui: &'a mut Ui,
    /// ID of the parent item in the logical tree.
    id: ElementID,
    /// Child visual elements
    children: Vec<VirtualElement>,
}

impl<'a> DomSink<'a>
{
    pub fn new(ui: &'a mut Ui) -> DomSink {
        DomSink {
            ui,
            id: 0,
            children: Vec::new()
        }
    }

    pub fn component<C, S, ChildrenFn, RenderFn>(&mut self, id: S, children: ChildrenFn, render: RenderFn) -> &mut VirtualElement
        where
            C: Component,
            S: Into<String>,
            RenderFn: FnOnce(&mut C, Vec<VirtualElement>, &mut DomSink),
            ChildrenFn: FnOnce(&mut DomSink)
    {
        // 1. collect children
        let id_str = id.into();
        let id = self.ui.id_stack.push_id(&id_str);
        let children = self.collect_children(id, children);

        // 2. render component
        let mut component: Box<C> = self.ui.get_component(id, || { Default::default() });
        render(&mut component, children, self);
        self.ui.insert_component(id, component);

        self.ui.id_stack.pop_id();
        self.children.last_mut().unwrap()
    }

    pub fn children(&self) -> &[VirtualElement]
    {
        &self.children[..]
    }

    pub fn collect_children(&mut self, id: ElementID, children: impl FnOnce(&mut DomSink)) -> Vec<VirtualElement>
    {
        let mut sink = DomSink {
            ui: &mut self.ui,
            children: Vec::new(),
            id
        };
        children(&mut sink);
        sink.into_elements()
    }

    pub fn push(&mut self, elements: Vec<VirtualElement>) {
        self.children.extend(elements);
    }

    /// Adds a text element.
    pub fn text<S: Into<String>>(&mut self, text: S) -> &mut VirtualElement {
        //self.children.push(Nde)
        let index = self.children.len();
        let id = self.ui.id_stack.push_id(&index);
        let vdom = VirtualElement::new_text(id, "text", text.into());
        self.children.push(vdom);
        self.ui.id_stack.pop_id();
        self.children.last_mut().unwrap()
    }

    /// Adds a div
    pub fn div(&mut self, class: &str, children: impl FnOnce(&mut DomSink)) -> &mut VirtualElement
    {
        // compute the ID of this element
        // item ID = child index.
        let index = self.children.len();
        let id = self.ui.id_stack.push_id(&index);
        let children = self.collect_children(id, children);
        let vdom = VirtualElement::new_div(id, class, children);
        self.children.push(vdom);
        self.ui.id_stack.pop_id();
        self.children.last_mut().unwrap()
    }

    pub fn into_elements(self) -> Vec<VirtualElement> {
        self.children
    }
}

