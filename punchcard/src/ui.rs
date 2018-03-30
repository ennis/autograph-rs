use nvg;
use std::collections::HashMap;
use autograph::rect_transform::*;
use std::hash::{Hash, Hasher, SipHasher};

pub struct FontId(u32);

pub enum DrawItem
{
    Text { text: String, font: FontId, pos: (f32,f32), opts: nvg::TextOptions },
    Rect { x: f32 , y: f32, w: f32, h: f32, radius: Option<f32>, fill: nvg::FillStyle, stroke: nvg::StrokeStyle },
}

pub struct ItemResult
{
    /// The item was clicked since the last call
    pub clicked: bool,
    /// The mouse is hovering over the item
    pub hover: bool
}

type ItemID = u64;

pub struct ItemBounds
{
    left: f32,
    right: f32,
    top: f32,
    bottom: f32
}

#[derive(Copy,Clone,Debug)]
pub struct Region
{
    /// Flexible
    layout: RectTransform,
    /// Computed item bounds
    bounds: ItemBounds
}


#[derive(Copy,Clone,Debug)]
pub enum Style
{
    Invisible,
    ButtonBackground,
    PanelBackground
}

#[derive(Copy,Clone,Debug)]
pub struct Item
{
    parent: ItemID,
    region: Region
}

pub struct Ui
{
    draw_items: Vec<DrawItem>,
    items: HashMap<ItemID, Item>,
    id_stack: Vec<ItemID>
}

// NONE of the methods should return a borrowed value (this will block everything)
// Of course, UI submission should be single-threaded
// layouts? nested things?
// begin/end pairs? closures?
impl Ui
{
    pub fn new() -> Ui {
        Ui {
            draw_items: Vec::new(),
            items: HashMap::new(),
            id_stack: Vec::new()
        }
    }

    fn chain_hash(&self, s: &str) -> ItemID {
        let stacklen = self.id_stack.len();
        let key1 = if stacklen >= 2 { self.id_stack[stacklen-2] } else { 0 };
        let key0 = if stacklen >= 1 { self.id_stack[stacklen-1] } else { 0 };
        let mut sip = SipHasher::new_with_keys(key0,key1);
        s.hash(sip);
        sip.finish()
    }

    pub fn push_id(&mut self, s: &str) -> ItemID {
        let id = self.chain_hash(s);
        self.id_stack.push(id);
        id
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    pub fn vbox<F: FnOnce(&mut Self)>(&mut self, f: F) -> ItemResult {
        // begin vbox
        // end vbox
        ItemResult { clicked: false, hover: false }
    }

    pub fn button<S: Into<String>>(&mut self, label: S) -> ItemResult {
        // get or create cache entry
        // restore state for widget

        // perform hit-testing
        // update animations
        // update style vars
        // redraw the item

        ItemResult { clicked: false, hover: false }
    }

    pub fn text<S: Into<String>>(&mut self, label: S) -> ItemResult {
        ItemResult { clicked: false, hover: false }
    }

    pub fn layout_and_render<'a>(&mut self, window_size: (u32,u32), frame: nvg::Frame<'a>) {

    }
}