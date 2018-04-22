use autograph::rect_transform::*;
use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
use cassowary::WeightedRelation::*;
use cassowary::{Solver, Variable};
use nvg;
use petgraph::graphmap::DiGraphMap;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher, SipHasher};

pub struct FontId(u32);

pub enum DrawItem {
    Text {
        text: String,
        font: FontId,
        pos: (f32, f32),
        opts: nvg::TextOptions,
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: Option<f32>,
        fill: nvg::FillStyle,
        stroke: nvg::StrokeStyle,
    },
}

pub struct ItemResult {
    /// The item was clicked since the last call
    pub clicked: bool,
    /// The mouse is hovering over the item
    pub hover: bool,
}

type ItemID = u64;

#[derive(Copy, Clone, Debug)]
pub struct ItemBounds {
    left: Variable,
    right: Variable,
    top: Variable,
    bottom: Variable,
}

impl Default for ItemBounds {
    fn default() -> ItemBounds {
        ItemBounds {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Style {
    Invisible,
    ButtonBackground,
    PanelBackground,
}

/// A widget.
#[derive(Clone)]
pub struct Item {
    /// Preferred size of the item
    /// They are automatically added as constraints, but you can add them manually.
    pref_width: Option<f32>,
    pref_height: Option<f32>,
    /// item bounds as cassowary variables
    bounds: ItemBounds,
    /// Custom state
    state: Box<RefCell<Any>>,
}

pub struct Ui {
    draw_items: Vec<DrawItem>,
    items: HashMap<ItemID, Item>,
    id_stack: Vec<ItemID>,
    root: ItemID,
    graph: DiGraphMap<ItemID, ()>,
    solver: Solver,
}

/// A layouter automatically adds constraints to the solver
/// as child items are added into a container
pub trait Layouter {
    // sometimes a layout can be calculated on-the-fly
    // returns none if cannot be computed now (deferred)
    // the item may not have a preferred size yet
    fn layout(&mut self, ui: &Ui, solver: &mut Solver, item: &Item, child: &Item, child_index: u32);

    // Needs access to all children
    // All children have a 'preferred size'
    // layouter mutates region.layout
    // it is only called if no explicit transform was specified
    fn deferred_layout(&mut self, ui: &Ui, item: &Item);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Padding {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

struct VboxLayouter {
    padding: Padding, // top, right, bottom, left or whatever
    spacing: f32,
    last_y_pos: Variable,
    distributed_height: Variable,
}

impl VboxLayouter {
    fn new() -> VboxLayouter {
        VboxLayouter {
            padding: Padding {
                left: 2.0,
                right: 2.0,
                top: 2.0,
                bottom: 2.0,
            },
            spacing: 2.0,
            last_y_pos: Variable::new(),
            distributed_height: Variable::new(),
        }
    }
}

impl Layouter for VboxLayouter {
    fn layout(
        &mut self,
        ui: &Ui,
        solver: &mut Solver,
        item: &Item,
        child: &Item,
        child_index: u32,
    ) {
        let pref_width = child.pref_width.borrow().clone();
        let pref_height = child.pref_height.borrow().clone();
        let bounds = child.bounds;
        let parent_bounds = item.bounds;

        // if a preferred width is set:
        // * right - left == pref_width
        // otherwise, stretch to parent container
        if let Some(pref_width) = pref_width {
            solver.add_constraints(&[
                bounds.left | EQ(STRONG) | parent_bounds.left + self.padding.left,
                bounds.right | EQ(STRONG) | bounds.left + pref_width,
            ]);
        } else {
            solver.add_constraints(&[
                bounds.left | EQ(STRONG) | parent_bounds.left + self.padding.left,
                bounds.right | EQ(STRONG) | parent_bounds.right - self.padding.right,
            ]);
        }

        // vertical layout
        // comes after the last element
        solver.add_constraints(&[
            // top == last_y_pos + spacing
            if child_index == 0 {
                bounds.top | EQ(STRONG) | parent_bounds.top + self.padding.top
            } else {
                bounds.top | EQ(STRONG) | self.last_y_pos + self.spacing
            },
        ]);

        //
        solver.add_constraints(&[if let Some(pref_height) = pref_height {
            bounds.bottom | EQ(STRONG) | bounds.top + pref_height
        } else {
            bounds.bottom | EQ(STRONG) | bounds.top + self.distributed_height
        }]);
    }

    fn deferred_layout(&mut self, ui: &Ui, item: &Item) {
        // Nothing to do
    }
}

// use cassowary for layout?
// child.top = prev.bottom + spacing
// child.right = parent.right
// child.left = parent.left
// child.bottom = preferred_height (weak)

// NONE of the methods should return a borrowed value (this will block everything)
// Of course, UI submission should be single-threaded
// layouts? nested things?
// begin/end pairs? closures?
//
// Custom layouts: need to be called back
impl Ui {
    pub fn new() -> Ui {
        let root: ItemID = 0;
        let mut graph = DiGraphMap::new();
        graph.add_node(root);
        Ui {
            draw_items: Vec::new(),
            items: HashMap::new(),
            id_stack: vec![root],
            root,
            graph,
            solver: Solver::new(),
        }
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> ItemID {
        let stacklen = self.id_stack.len();
        let key1 = if stacklen >= 2 {
            self.id_stack[stacklen - 2]
        } else {
            0
        };
        let key0 = if stacklen >= 1 {
            self.id_stack[stacklen - 1]
        } else {
            0
        };
        let mut sip = SipHasher::new_with_keys(key0, key1);
        s.hash(&mut sip);
        sip.finish()
    }

    pub fn push_id<H: Hash>(&mut self, s: &H) -> ItemID {
        let id = self.chain_hash(s);
        let parent_id = *self.id_stack.last().unwrap();
        self.id_stack.push(id);
        // add a new edge
        // XXX should we always add a node here? what about dummy IDs in the stack?
        // should probably be push_item or something like that
        self.graph.add_node(id);
        self.graph.add_edge(parent_id, id, ());
        debug!("ID stack -> {:?}", self.id_stack);
        id
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    pub fn vbox<S: Into<String>, F: FnOnce(&mut Self)>(&mut self, id: S, f: F) -> ItemResult {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.push_id(&id_str);
        // state
        struct VBoxState {}
        // if item not present, create it
        self.items.entry(id).or_insert_with(|| Item {
            pref_width: RefCell::new(None),
            pref_height: RefCell::new(None),
            bounds: Default::default(),
            state: Box::new(RefCell::new(VBoxState)),
        });
        // insert children
        f(self);
        let result = ItemResult {
            clicked: false,
            hover: false,
        };
        self.pop_id();
        result
    }

    pub fn button<S: Into<String>>(&mut self, label: S) -> ItemResult {
        let label_str = label.into();
        let id = self.push_id(&label_str);
        struct ButtonState {}
        self.items.entry(id).or_insert_with(|| Item {
            pref_width: RefCell::new(None),
            pref_height: RefCell::new(None),
            bounds: Default::default(),
            state: Box::new(RefCell::new(ButtonState)),
        });

        // create the text label
        self.text(label_str);
        let result = ItemResult {
            clicked: false,
            hover: false,
        };
        self.pop_id();
        result
    }

    pub fn text<S: Into<String>>(&mut self, label: S) -> ItemResult {
        let label_str = label.into();
        let id = self.push_id(&label_str);
        // create an item
        self.items.insert(
            id,
            Item {
                layout: Default::default(),
                bounds: None,
            },
        );
        let result = ItemResult {
            clicked: false,
            hover: false,
        };
        self.pop_id();
        result
    }

    pub fn layout_and_render<'a>(&mut self, window_size: (u32, u32), frame: nvg::Frame<'a>) {
        /* for n in self.graph.neighbors(self.root) {
            // the iteration borrows self, cannot call any other mut function here
        }*/
    }

    // TODO custom layout constraints
}
