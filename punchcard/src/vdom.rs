use super::*;
use std::fmt::Debug;

/// Bits of DOM.
#[derive(Debug, Clone)]
pub enum Contents<T>
{
    Div(Vec<T>),
    Text(String)
}

/// Data shared between the actual DOM and the VDOM.
#[derive(Debug, Clone)]
pub struct Element<T: Debug + Clone>
{
    id: ElementID,
    class: String,
    contents: Contents<Element<T>>,
    extra: T
    //layout_overrides: LayoutOverrides,
}

pub type VirtualElement = Element<()>;
pub type RetainedElement = Element<RetainedData>;

/// Actual DOM node (persistent across frames).
#[derive(Debug, Clone)]
pub struct RetainedData
{
    /// Cached calculated layout.
    layout: Layout,
    /// yoga Flexbox node
    flex: yoga::Node,
}

impl VirtualElement
{
    pub fn new_text<S: Into<String>>(id: ElementID, class: &str, text: S) -> VirtualElement
    {
        VirtualElement {
            id,
            class: class.into(),
            //layout_overrides: Default::default(),
            contents: Contents::Text(text.into()),
            extra: ()
        }
    }

    pub fn new_div(id: ElementID, class: &str, children: Vec<VirtualElement>) -> VirtualElement
    {
        VirtualElement {
            id,
            class: class.into(),
            //layout_overrides: Default::default(),
            contents: Contents::Div(children),
            extra: ()
        }
    }

    pub fn into_retained(self) -> RetainedElement
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
            extra: RetainedData {
                layout: Layout::default(),
                flex
            }
        }
    }

}

impl RetainedElement
{
    /// Unconditionally overwrite the node with
    pub fn overwrite(&mut self, vdom: VirtualElement)
    {
        self.id = vdom.id;
        self.class = vdom.class;

    }

    /// Update in place.
    pub fn update(&mut self, vdom: VirtualElement) {
        // compare IDs
        if self.id != vdom.id {
            self.overwrite(vdom);
        }
    }

    pub fn update_child(&mut self, vdom: VirtualElement) {

        match self.contents {
            Contents::Text(s) => {
                // replace everything
            },

        }

    }
}

pub fn compare_element_list(parent: &mut RetainedElement, retained: &mut Vec<RetainedElement>, vdom: &mut Vec<VirtualElement>)
{
    let num_elem = vdom.len();
    'outer: for (vi,v) in vdom.drain(..).enumerate() {
        // the first vi elements are already updated.
        for ri in vi..retained.len() {
            if retained[ri].id == v.id {
                // matching node, update in place
                retained[ri].update(v);
                // swap flex nodes
                parent.extra.flex.remove_child(&mut retained[ri].extra.flex);
                parent.extra.flex.insert_child(&mut retained[ri].extra.flex, vi);
                // swap at the correct position
                retained.swap(ri, vi);
                continue 'outer;
            }
        }
        // no matching element found in retained graph: insert a new one
        let mut new = v.into_retained();
        parent.extra.flex.insert_child(&mut new.extra.flex, vi);
        retained.push(v.into_retained());
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
        let children = {
            let mut sink = DomSink {
                ui: &mut self.ui,
                children: Vec::new(),
                id
            };

            children(&mut sink);
            sink.children
        };
        children
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
}

