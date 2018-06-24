use super::*;
use super::style::{apply_to_flex_node, Styles};

use std::fmt::Debug;
use std::rc::Rc;

/// Bits of DOM.
#[derive(Debug, Eq, PartialEq)]
pub enum Contents
{
    Element,
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

/// Actual DOM node (persistent across frames).
#[derive(Debug)]
pub struct RetainedNode
{
    pub(super) id: ElementID,
    pub(super) class: String,
    /// Cached calculated layout.
    pub(super) layout: Layout,
    /// Styles must be recalculated.
    pub(super) styles_dirty: bool,
    /// Resolved styles.
    pub(super) styles: Option<Rc<Styles>>,
    /// yoga Flexbox node
    pub(super) flex: yoga::Node,
    pub(super) contents: Contents,
    pub(super) layout_overrides: LayoutOverrides,
}

impl RetainedNode
{
    pub(super) fn update_layout(&mut self, parent_layout: &Layout) -> Layout
    {
        let layout = Layout::from_yoga_layout(parent_layout, self.flex.get_layout());
        self.layout = layout;
        //debug!("calc layout: {:?}", layout);
        layout
    }
}

/// Virtual DOM node.
pub struct VirtualNode
{
    pub(super) id: ElementID,
    pub(super) class: String,
    pub(super) layout_overrides: LayoutOverrides,
    pub(super) contents: Contents,
    pub(super) children: Vec<VirtualNode>
}

impl VirtualNode
{
    pub fn new_text<S: Into<String>>(id: ElementID, class: &str, text: S) -> VirtualNode
    {
        VirtualNode {
            id,
            class: class.into(),
            layout_overrides: Default::default(),
            contents: Contents::Text(text.into()),
            children: Vec::new()
        }
    }

    pub fn new_element(id: ElementID, class: &str, children: Vec<VirtualNode>) -> VirtualNode
    {
        VirtualNode {
            id,
            class: class.into(),
            layout_overrides: Default::default(),
            contents: Contents::Element,
            children
        }
    }

    pub(super) fn into_retained(mut self, arena: &mut Arena<RetainedNode>, parent: Option<NodeId>) -> NodeId
    {
        let mut flex = yoga::Node::new();

        if let Some(parent) = parent
        {
            let flex_parent = &mut arena[parent].data_mut().flex;
            let child_count = flex_parent.child_count();
            flex_parent.insert_child(&mut flex, child_count);
        }

        let node = RetainedNode {
            id: self.id,
            class: self.class,
            contents: self.contents,
            layout_overrides: self.layout_overrides,
            layout: Layout::default(),
            styles: None,
            styles_dirty: true,
            flex
        };

        // add to parent
        let node_id = arena.new_node(node);
        if let Some(parent) = parent {
            parent.append(node_id, arena);
        }

        // recursively add children
        for c in self.children.drain(..) {
            c.into_retained(arena, Some(node_id));
        }

        node_id
    }
}

pub(super) fn update_node(arena: &mut Arena<RetainedNode>, id: NodeId, vn: VirtualNode)
{
    let recreate = match (&vn.contents, &arena[id].data().contents) {
        (&Contents::Element, &Contents::Element) => false,
        (&Contents::Text(_), &Contents::Text(_)) => false,
        _ => true
    };

    if recreate {
        // drop the node and create another
        let parent = arena[id].parent();
        arena.remove_node(id);
        //if let Some(parent) = parent {
        vn.into_retained(arena, parent);
        //}
    }
    else {
        // update in place
        let this = &mut arena[id];
        let data = this.data_mut();
        if data.class != vn.class {
            data.styles_dirty = true;
            data.class = vn.class;
        }
        if data.contents != vn.contents {
            data.contents = vn.contents;
        }
    }
}

/// Update children in place.
/// We need to look for matching IDs between the retained and the virtual DOM.
/// Even if the position does not match, we must keep internal state for child elements with the same ID.
/// Reordering children should not reset their internal states.
fn update_children(arena: &mut Arena<RetainedNode>, parent: NodeId, mut vdom: Vec<VirtualNode>)
{
    //debug!("update_element_list retained={:#?}, vdom={:#?}", retained, vdom);
    let num_elem = vdom.len();
    let mut next = arena[parent].first_child();

    'outer: for (vi,vn) in vdom.drain(..).enumerate() {
        if let Some(n) = next {
            update_node(arena, n, vn);
            next = arena[n].next_sibling();
        } else {
            vn.into_retained(arena, Some(parent));
        }
    }

    // drop remaining nodes.
    while let Some(n) = next {
        next = arena[n].next_sibling();
        arena.remove_node(n);
    }
}

pub struct DomSink<'a>
{
    /// Ref to the root UI object.
    ui: &'a mut Ui,
    /// ID of the parent item in the logical tree.
    id: ElementID,
    /// Child visual elements
    children: Vec<VirtualNode>,
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

    pub fn component<C, S, ChildrenFn, RenderFn>(&mut self, id: S, children: ChildrenFn, render: RenderFn) -> &mut VirtualNode
        where
            C: Component,
            S: Into<String>,
            RenderFn: FnOnce(&mut C, Vec<VirtualNode>, &mut DomSink),
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

    pub fn children(&self) -> &[VirtualNode]
    {
        &self.children[..]
    }

    pub fn collect_children(&mut self, id: ElementID, children: impl FnOnce(&mut DomSink)) -> Vec<VirtualNode>
    {
        let mut sink = DomSink {
            ui: &mut self.ui,
            children: Vec::new(),
            id
        };
        children(&mut sink);
        sink.into_nodes()
    }

    pub fn push(&mut self, nodes: Vec<VirtualNode>) {
        self.children.extend(nodes);
    }

    /// Adds a text element.
    pub fn text<S: Into<String>>(&mut self, text: S) -> &mut VirtualNode {
        //self.children.push(Nde)
        let index = self.children.len();
        let id = self.ui.id_stack.push_id(&index);
        let vdom = VirtualNode::new_text(id, "text", text.into());
        self.children.push(vdom);
        self.ui.id_stack.pop_id();
        self.children.last_mut().unwrap()
    }

    /// Adds a div
    pub fn div(&mut self, class: &str, children: impl FnOnce(&mut DomSink)) -> &mut VirtualNode
    {
        // compute the ID of this element
        // item ID = child index.
        let index = self.children.len();
        let id = self.ui.id_stack.push_id(&index);
        let children = self.collect_children(id, children);
        let vdom = VirtualNode::new_element(id, class, children);
        self.children.push(vdom);
        self.ui.id_stack.pop_id();
        self.children.last_mut().unwrap()
    }

    pub fn into_nodes(self) -> Vec<VirtualNode> {
        self.children
    }
}

/// DOM helper macros.
#[macro_export]
macro_rules! dom {
    // leaf DOM nodes with no params
    ($sink:expr; @$component:ident ; ) => { $component($sink); };
    ($sink:expr; @$component:ident ; $($rest:tt)* ) => { $component($sink); dom!($sink; $($rest)*); };
    // leaf DOM nodes
    ($sink:expr; @$component:ident ( $($e:expr),* ) ; ) => { $component($sink, $($e,)*); };
    ($sink:expr; @$component:ident ( $($e:expr),* ) ; $($rest:tt)* ) => { $component($sink, $($e,)*); dom!($sink; $($rest)*); };
    // inner nodes with no params
    ($sink:expr; @$component:ident {  $($body:tt)* }) => { $component($sink, |sink| { dom!(sink; $($body)*); }); };
    ($sink:expr; @$component:ident {  $($body:tt)* } $($rest:tt)* ) => { $component($sink, |sink| { dom!(sink; $($body)*); }); dom!($sink; $($rest)*); };
    // inner nodes
    ($sink:expr; @$component:ident ( $($e:expr),* ) {  $($body:tt)* }) => { $component($sink, $($e,)* |sink| { dom!(sink; $($body)*); }); };
    ($sink:expr; @$component:ident ( $($e:expr),* ) {  $($body:tt)* } $($rest:tt)* ) => { $component($sink, $($e,)* |sink| { dom!(sink; $($body)*); }); dom!($sink; $($rest)*); };
    // statements
    ($sink:expr; $head:stmt ; $($rest:tt)* ) => { $head; $($rest:tt)* };
}
