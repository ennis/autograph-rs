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

    pub fn set_width(&mut self, width: f32) -> &mut Self
    {
        self.layout_overrides.width = Some(width.point());
        self
    }

    pub fn set_height(&mut self, height: f32) -> &mut Self
    {
        self.layout_overrides.height = Some(height.point());
        self
    }

    pub fn set_size(&mut self, size: (f32,f32)) -> &mut Self
    {
        self.layout_overrides.width = Some(size.0.point());
        self.layout_overrides.height = Some(size.1.point());
        self
    }

    pub fn set_position(&mut self, pos: (f32,f32)) -> &mut Self
    {
        self.layout_overrides.left = Some(pos.0.point());
        self.layout_overrides.top = Some(pos.1.point());
        self
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
         debug!("RECREATE");
        // drop the node and create another
        let parent = arena[id].parent();
        arena.remove_node(id);
        //if let Some(parent) = parent {
        vn.into_retained(arena, parent);
        //}
    }
    else {
        // update in place
        {
            let this = &mut arena[id];
            let data = this.data_mut();
            if data.class != vn.class {
                data.styles_dirty = true;
                data.class = vn.class;
            }
            if data.contents != vn.contents {
                data.contents = vn.contents;
            }
            if data.layout_overrides != vn.layout_overrides {
                //debug!("LAYOUT OVERRIDE {:?} -> {:?}", data.layout_overrides, vn.layout_overrides);
                data.layout_overrides = vn.layout_overrides;
            }
        }
        update_children(arena, id, vn.children);
        //data.id = vn.id;
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

    ///
    /// Instantiates a component which has no children.
    pub fn component<C, S, R, RenderFn>(&mut self, id: S, component_init: C, render_fn: RenderFn) -> R
        where
            C: Component+Default,
            S: Into<String>,
            RenderFn: FnOnce(&mut C, &mut DomSink) -> R
    {
        self.aggregate_component(id, component_init, |_|{}, move |state,children,dom| {
            render_fn(state,dom)
        })
    }

    ///
    /// Instantiates a component with a list of children.
    /// The child elements can be manipulated in render().
    /// TODO bikeshed the name.
    pub fn aggregate_component<C, S, R, ChildrenFn, RenderFn>(&mut self, id: S, component_init: C, children_fn: ChildrenFn, render_fn: RenderFn) -> R
        where
            C: Component+Default,
            S: Into<String>,
            RenderFn: FnOnce(&mut C, Vec<VirtualNode>, &mut DomSink) -> R,
            ChildrenFn: FnOnce(&mut DomSink)
    {
        // 1. collect children
        let id_str = id.into();
        let id = self.ui.id_stack.push_id(&id_str);
        let (_,children) = self.collect_children(id, children_fn);

        // 2. render component
        let mut component = self.ui.get_component::<C,_>(id, move || { component_init });
        let (render_result, mut rendered) = {
            let c = component.as_mut_any().downcast_mut().expect("unexpected component type");
            let res = self.collect_children(id, |dom| {
                render_fn(c, children, dom)
            });
            c.post_frame();
            res
            // drop borrow of component through component_ref
        };
        assert!(rendered.len() <= 1, "A component cannot render more than one element (rendered.len() = {})", rendered.len());
        // create vdom node for component
        // let class empty because it's a wrapper node.
        if rendered.len() == 1 {
            let mut vdom = rendered.pop().unwrap();
            // HACK: correct the ID of the vdom node so that it matches the one of the component.
            vdom.id = id;
            self.children.push(vdom);
        }
        self.ui.insert_component(id, component);
        self.ui.id_stack.pop_id();
        render_result
    }

    pub fn children(&self) -> &[VirtualNode]
    {
        &self.children[..]
    }

    pub fn collect_children<R, F>(&mut self, id: ElementID, children_fn: F) -> (R, Vec<VirtualNode>)
    where
        F: FnOnce(&mut DomSink) -> R
    {
        let mut sink = DomSink {
            ui: &mut self.ui,
            children: Vec::new(),
            id
        };
        let result = children_fn(&mut sink);
        (result, sink.into_nodes())
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
        let (_,children) = self.collect_children(id, children);
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
