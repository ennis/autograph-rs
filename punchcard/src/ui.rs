use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
use cassowary::WeightedRelation::*;
use cassowary::{Solver, Variable, Constraint};
use nvg;
use petgraph::graphmap::DiGraphMap;
use std::any::Any;
use std::cell::{Cell,RefCell};
use std::collections::{hash_map, HashMap};
use std::hash::{Hash, Hasher, SipHasher};
use std::mem::replace;
use diff;
use indexmap::{IndexMap, map::Entry, map::VacantEntry, map::OccupiedEntry};


// Layout must be done once the full list of children is known
// Detect differences in the list of children => produce diff
// prev, next
// on insertion at child index #n:
// if prev[#n].id != id { add diff: Replace(#0 -> id) }
// remaining indices { Remove(#n) }
// new indices { Add(#n) }
// process all diffs:
// if replace or remove, remove all constraints from the removed items, move items in the garbage bin (or simply delete them)
// perform layout with all added items
//
// Context for adding an element:
// prev_list, diffs, elem_hash, child_index
// prev_list: list of item IDs
//
// Context for layout:
// - child_list: Vec<ID>
// - added_items: Vec<ID>
//
// Issue: where to store the list of children for an item?
// -> child list is borrowed during updates, which borrows the hashmap, which prevents mutation of the hashmap
// -> this is safe however, since the item owning the list cannot be deleted
// -> solutions: - hashmap lookup of the parent each time the child list needs to be accessed: costly
//               - unsafe code**
//               - Rc<Item>
//               - each item has its own hashmap
//               - move the boxed data outside, and put it back again

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
    //pref_width: Option<f32>,
    //pref_height: Option<f32>,
    /// item bounds as cassowary variables
    bounds: ItemBounds,
    /// cassowary constraints associated to this item
    constraints: Vec<Constraint>,
    //state: Box<RefCell<Any>>,
    children: IndexMap<ItemID, Item>,
    /// Last update frame index
    last_update: u64,
    needs_relayout: Cell<bool>
}

impl Item
{
    pub fn new(cur_frame: u64) -> Item {
        Item {
            children: IndexMap::new(),
            last_update: cur_frame,
            bounds: ItemBounds::default(),
            constraints: Vec::new(),
            needs_relayout: Cell::new(true)
        }
    }

    pub fn replace_children(&mut self, new: IndexMap<ItemID, Item>) -> IndexMap<ItemID, Item> {
        replace(&mut self.children, new)
    }

    pub fn add_constraint(&mut self, solver: &mut Solver, constraint: Constraint) {
        self.constraints.push(constraint.clone());
        solver.add_constraint(constraint);
    }

    pub fn add_constraints<'a, I: IntoIterator<Item = &'a Constraint>>(&mut self, solver: &mut Solver, constraints: I) {
        for constraint in constraints {
            self.add_constraint(solver, constraint.clone());
        }
    }

    pub fn remove_all_constraints(&mut self, solver: &mut Solver) {
        for c in self.constraints.drain(..) {
            solver.remove_constraint(&c);
        }
    }
}

pub struct IdStack(Vec<ItemID>);

impl IdStack
{
    pub fn new(root_id: ItemID) -> IdStack
    {
        IdStack(vec![root_id])
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> ItemID {
        let stacklen = self.0.len();
        let key1 = if stacklen >= 2 {
            self.0[stacklen - 2]
        } else {
            0
        };
        let key0 = if stacklen >= 1 {
            self.0[stacklen - 1]
        } else {
            0
        };
        let mut sip = SipHasher::new_with_keys(key0, key1);
        s.hash(&mut sip);
        sip.finish()
    }

    pub fn push_id<H: Hash>(&mut self, s: &H) -> ItemID {
        let id = self.chain_hash(s);
        let parent_id = *self.0.last().unwrap();
        self.0.push(id);
        id
    }

    pub fn pop_id(&mut self) {
        self.0.pop();
    }
}

pub struct Ui {
    id_stack: IdStack,
    root: Item,
    cur_frame: u64,
    solver: Solver,
    calc_layout: HashMap<Variable, f32>
}

// - cannot have a mutable borrow of the Ui (for the ID stack and the solver) and a mutable borrow of
//   one Item (borrows Ui indirectly) at the same time.
//   -> separate the Item borrow chain from Ui

pub struct UiContainer<'a> {
    id_stack: &'a mut IdStack,
    solver: &'a mut Solver,
    item: &'a mut Item,
    cur_frame: u64,
    cur_index: usize
}

impl<'a> UiContainer<'a>
{
    pub fn new_item<'s, F: FnOnce()->Item>(&'s mut self, new_item_id: ItemID, f: F) -> UiContainer<'s>  {

        // when inserting a child item:
        //      - if index matches: OK
        //      - else: swap the item at the correct location, mark it for relayout
        //              - insert item at the end, swap_remove item at index, put item back
        let cur_index = self.cur_index;
        let item_reinsert = {
            let entry = self.item.children.entry(new_item_id);
            // TODO accelerate common case (no change) by looking by index first
            match entry {
                Entry::Vacant(ref entry) => {
                    // entry is vacant: must insert at current location
                    Some(f())
                },
                Entry::Occupied(entry) => {
                    let index = entry.index();
                    // if the child item exists, see if its index corresponds
                    if cur_index != index {
                        // item has moved: extract the item from its previous position
                        Some(entry.remove())
                    } else {
                        // child item exists and has not moved
                        entry.get().needs_relayout.set(false);
                        None
                    }
                }
            }
            // drop borrow by entry()
        };

        if let Some(item) = item_reinsert {
            // must insert or reinsert an item at the correct index
            // mark item for relayout
            // insert last
            item.needs_relayout.set(true);
            let len = self.item.children.len();
            self.item.children.insert(new_item_id, item);
            if cur_index != len {
                // we did not insert it at the correct position: need to swap the newly inserted element in place
                // remove element at target position: last inserted item now in position
                let kv = self.item.children.swap_remove_index(cur_index).unwrap();
                // reinsert previous item
                self.item.children.insert(kv.0, kv.1);
                debug!("item {:016X} moved to {}", new_item_id, cur_index);
            } else {
                debug!("item {:016X} inserted at {}", new_item_id, cur_index);
            }
        } else {
            //debug!("item {} at {} did not move", new_item_id, cur_index);
        }

        let new_item = self.item.children.get_index_mut(cur_index).unwrap().1;

        self.cur_index += 1;

        UiContainer {
            id_stack: self.id_stack,
            solver: self.solver,
            item: new_item,
            cur_frame: self.cur_frame,
            cur_index: 0
        }
    }

    /// Remove items that are not present anymore
    /*pub fn finalize(&mut self)
    {
        let cur_index = self.cur_index;
        //self.item.children.retain(|(i,| )
    }*/

    pub fn new(item: &'a mut Item, id_stack: &'a mut IdStack, solver: &'a mut Solver, cur_frame: u64) -> UiContainer<'a> {
        UiContainer {
            id_stack,
            item,
            solver,
            cur_frame,
            cur_index: 0
        }
    }
}


fn vertical_layout(id: ItemID, item: &mut Item, solver: &mut Solver)
{
    // iterate over the child items
    // can skip layout if no modification was made
    let needs_relayout = item.children.values().any(|item| item.needs_relayout.get());
    if !needs_relayout { return }

    debug!("RELAYOUT {:016X}", id);
    // do full relayout (could be optimized...)
    let mut cur_y_pos = item.bounds.top;
    let len = item.children.len();
    let parent_bounds = item.bounds;
    for (i,child) in item.children.values_mut().enumerate() {
        let is_last = i == len-1;
        let bounds = child.bounds;
        child.remove_all_constraints(solver);
        child.add_constraints(solver, &[
            bounds.top |EQ(STRONG)| cur_y_pos,
            bounds.bottom - bounds.top |EQ(WEAK)| 70.0,
            bounds.left |EQ(STRONG)| parent_bounds.left,
            bounds.right |EQ(STRONG)| parent_bounds.right]);
        cur_y_pos = bounds.bottom;
        // last element fills remaining available space
        if is_last {
            child.add_constraint(solver, bounds.bottom |EQ(STRONG)| parent_bounds.bottom);
        }
    }
}




/// A layouter automatically adds constraints to the solver
/// as child items are added into a container
/*pub trait Layouter {
    // sometimes a layout can be calculated on-the-fly
    // returns none if cannot be computed now (deferred)
    // the item may not have a preferred size yet
    //fn layout(&mut self, ui: &Ui, solver: &mut Solver, item: &Item, child: &Item, child_index: u32);

    // Needs access to all children
    // All children have a 'preferred size'
    // layouter mutates region.layout
    // it is only called if no explicit transform was specified
    //fn deferred_layout(&mut self, ui: &Ui, item: &Item);
}*/

#[derive(Copy, Clone, Debug)]
struct Padding {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

/*struct VboxLayouter {
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
        let pref_width = child.pref_width;
        let pref_height = child.pref_height;
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
}*/

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
        let mut solver = Solver::new();
        let mut root = Item::new(0);
        solver.add_edit_variable(root.bounds.bottom, STRONG).unwrap();
        solver.add_edit_variable(root.bounds.top, STRONG).unwrap();
        solver.add_edit_variable(root.bounds.left, STRONG).unwrap();
        solver.add_edit_variable(root.bounds.right, STRONG).unwrap();

        let mut ui = Ui {
            id_stack: IdStack::new(0),
            root,
            cur_frame: 0,
            solver,
            calc_layout: HashMap::new()
        };
        ui
    }

    pub fn root<F: FnOnce(&mut UiContainer)>(&mut self, f: F)
    {
        let root = &mut self.root;
        let solver = &mut self.solver;
        let id_stack = &mut self.id_stack;
        let mut container = UiContainer::new(root, id_stack, solver, self.cur_frame);
        f(&mut container);
        vertical_layout(0, container.item, container.solver);
    }

    pub fn layout(&mut self, window_width: f32, window_height: f32) {
        self.solver.suggest_value(self.root.bounds.top, 0.0).unwrap();
        self.solver.suggest_value(self.root.bounds.bottom, window_height as f64).unwrap();
        self.solver.suggest_value(self.root.bounds.left, 0.0).unwrap();
        self.solver.suggest_value(self.root.bounds.right, window_width as f64).unwrap();

        let mut changes = self.solver.fetch_changes();
        if !changes.is_empty() {
            debug!("layout changes: {:?}", changes);
        }

        for change in changes {
            self.calc_layout.insert(change.0, change.1 as f32);
        }
    }

    pub fn render(&self, frame: &nvg::Frame)
    {
        self.render_item(&self.root, frame);
    }

    pub fn render_item(&self, item: &Item, frame: &nvg::Frame)
    {
        let top = self.solver.get_value(item.bounds.top) as f32;
        let bottom = self.solver.get_value(item.bounds.bottom) as f32;
        let left = self.solver.get_value(item.bounds.left) as f32;
        let right = self.solver.get_value(item.bounds.right) as f32;
        let w = right-left;
        let h = bottom-top;
        frame.path(
            |path| {
                path.rect((top, left), (w, h));
               /* path.fill(nvg::FillStyle {
                    coloring_style: nvg::ColoringStyle::Color(nvg::Color::new(
                        0.2, 0.2, 0.2, 0.7,
                    )),
                    ..Default::default()
                });*/
                path.stroke(nvg::StrokeStyle {
                    coloring_style: nvg::ColoringStyle::Color(nvg::Color::new(
                        0.5, 0.5, 0.5, 1.0,
                    )),
                    width: 1.0,
                    ..Default::default()
                });
            },
            Default::default(),
        );

        for child in item.children.values() {
            self.render_item(child, frame);
        }
    }
}

impl<'a> UiContainer<'a>
{
    // Check if the item at the current index in the previous list corresponds to the given ItemID
    // if not, the previous item is recycled (deleted or moved into the cache)
    // and f is called to create a new item: the new item is placed into the hashmap
    // Returns a reference to the item (locks the UiContainer).
    /*pub fn update_child<'b:'a, F: Fn() -> Item>(&'b mut self, item_id: ItemID, f: F) -> &'b mut Item
    {
        // more items this time
        if self.cur_index >= self.prev_list.len() {
            match self.ui.items.entry(item_id) {
                Entry::Occupied(ref mut entry) => {
                    // item has moved in the list
                    let item = entry.get_mut();
                    item.last_update = self.ui.cur_frame;
                    item
                },
                Entry::Vacant(ref mut entry) => {
                    // new item
                    let new = f();
                    entry.insert(new)
                }
            }
        }
        else {
            if self.prev_list[self.cur_index] != item_id {
                // item ID does not match with the previous item at this position in the list:
                // replace previous element
                match self.ui.items.entry(item_id) {
                    Entry::Occupied(ref mut entry) => {
                        // item has moved in the list
                        let item = entry.get_mut();
                        item.last_update = self.ui.cur_frame;
                        item
                    },
                    Entry::Vacant(ref mut entry) => {
                        entry.insert(new)
                    }
                }
            } else {
                // previous item ID matches: no update!
                let item = self.ui.items.get_mut(&item_id).expect("no matching item");
                item.last_update = self.ui.cur_frame;
                item
            }
        }
    }*/

    pub fn vbox<S: Into<String>, F: FnOnce(&mut UiContainer)>(
        &mut self,
        id: S,
        f: F,
    ) -> ItemResult {
        let cur_frame = self.cur_frame;
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.id_stack.push_id(&id_str);

        // insert children
        {
            let mut container = self.new_item(id, || { Item::new(cur_frame) } );
            f(&mut container);
            // apply layout
            vertical_layout(id, container.item, container.solver);
        }

        let result = ItemResult {
            clicked: false,
            hover: false,
        };
        self.id_stack.pop_id();
        result
    }
}
