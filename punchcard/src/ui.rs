use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
use cassowary::WeightedRelation::*;
use cassowary::{Solver, Variable, Constraint};
use nvg;
use petgraph::graphmap::DiGraphMap;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, hash_map::Entry, HashSet};
use std::hash::{Hash, Hasher, SipHasher};
use std::mem::replace;
use diff;
use indexmap::IndexMap;

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
    last_update: u64
}

impl Item
{
    pub fn new(cur_frame: u64) -> Item {
        Item {
            children: IndexMap::new(),
            last_update: cur_frame,
            bounds: ItemBounds::default(),
            constraints: Vec::new()
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
}

// - cannot have a mutable borrow of the Ui (for the ID stack and the solver) and a mutable borrow of
//   one Item (borrows Ui indirectly) at the same time.
//   -> separate the Item borrow chain from Ui

pub struct UiContainer<'a> {
    id_stack: &'a mut IdStack,
    solver: &'a mut Solver,
    item: &'a mut Item,
    cur_frame: u64
}

impl<'a> UiContainer<'a>
{
    pub fn new(item: &'a mut Item, id_stack: &'a mut IdStack, solver: &'a mut Solver, cur_frame: u64) -> UiContainer<'a> {
        UiContainer {
            id_stack,
            item,
            solver,
            cur_frame
        }
    }

    pub fn new_item<'s, F: FnOnce()->Item>(&'s mut self, new_item_id: ItemID, f: F) -> UiContainer<'s>  {
        let new_item = self.item.children.entry(new_item_id).or_insert_with(f);
        UiContainer {
            id_stack: self.id_stack,
            solver: self.solver,
            item: new_item,
            cur_frame: self.cur_frame
        }
    }
}

// layout: list of insertion/removals
//
/*fn vertical_layout(items: &HashMap<ItemID, RefCell<Item>>, solver: &mut Solver, parent_bounds: &ItemBounds, diff: &[diff::Result<&ItemID>])
{
    // Left: removed, Both: Same, Right: added
    // state: prev_vertical_var
    let mut cur_y = parent_bounds.top;
    let mut prev_item_changed = false;
    for (i,d) in diff.iter().enumerate() {
        let is_last = i == diff.len()-1;
        match d {
            diff::Result::Left(id_removed) => {
                let mut item = items.get(id_removed).expect("id not found").borrow_mut();
                debug!("Removing constraints for {}", id_removed);
                // remove all constraints associated to this item
                item.remove_all_constraints(solver);
                // signal that the next item needs to update its constraints
                prev_item_changed = true;
            },
            diff::Result::Both(id, _) => {
                let mut item = items.get(id).expect("id not found").borrow_mut();
                if prev_item_changed {
                    debug!("Updating constraints for {}", id);
                    // prev item changed, must relayout
                    item.remove_all_constraints(solver);
                    item.add_constraints(solver, &[
                        item.bounds.top |EQ(STRONG)| cur_y,
                        item.bounds.bottom - item.bounds.top |EQ(WEAK)| 70.0,
                        item.bounds.left |EQ(STRONG)| parent_bounds.left,
                        item.bounds.right |EQ(STRONG)| parent_bounds.right]);
                }
                cur_y = item.bounds.bottom;
            },
            diff::Result::Right(id_added) => {
                let mut item = items.get(id_added).expect("id not found").borrow_mut();
                debug!("Adding constraints for {}", id_added);
                item.remove_all_constraints(solver);
                item.add_constraints(solver, &[
                    item.bounds.top |EQ(STRONG)| cur_y,
                    item.bounds.bottom - item.bounds.top |EQ(WEAK)| 70.0,
                    item.bounds.left |EQ(STRONG)| parent_bounds.left,
                    item.bounds.right |EQ(STRONG)| parent_bounds.right]);
                cur_y = item.bounds.bottom;
                prev_item_changed = true;
                if is_last {
                    item.add_constraint(solver, item.bounds.bottom |EQ(WEAK)| parent_bounds.bottom);
                }
            }
        }
    }
}*/




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
        let mut ui = Ui {
            id_stack: IdStack::new(0),
            root: Item::new(0),
            cur_frame: 0,
            solver: Solver::new()
        };
        ui
    }

    pub fn root<F: FnOnce(&mut UiContainer)>(&mut self, f: F)
    {
        let root = &mut self.root;
        let solver = &mut self.solver;
        let id_stack = &mut self.id_stack;
        let mut root_container = UiContainer::new(root, id_stack, solver, self.cur_frame);
        f(&mut root_container);
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

            let mut container = self.new_item(id, || { debug!("new item"); Item::new(cur_frame) } );
            f(&mut container);
        }

        let result = ItemResult {
            clicked: false,
            hover: false,
        };
        self.id_stack.pop_id();
        result
    }
}
