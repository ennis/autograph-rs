// cassowary is too slow unfortunately: took 693ms to layout 100 items in a vbox
// good for static layouts, not so much for imgui context with a highly dynamic item count
// use yoga instead
use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
use cassowary::WeightedRelation::*;
use cassowary::{Solver, Variable, Constraint};

use yoga;
use yoga::prelude::*;
use yoga::FlexStyle::*;
use yoga::StyleUnit::{Auto, UndefinedValue};
use nvg;
use petgraph::graphmap::DiGraphMap;
use std::any::Any;
use std::cell::{Cell,RefCell};
use std::collections::{hash_map, HashMap};
use std::hash::{Hash, Hasher, SipHasher};
use std::mem::replace;
use diff;
use indexmap::{IndexMap, map::Entry, map::VacantEntry, map::OccupiedEntry};
use time;
use glutin::{WindowEvent, KeyboardInput, VirtualKeyCode};


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
    Nothing,
    Text(String),
    Rect,
}

pub struct ItemResult {
    /// The item was clicked since the last call
    pub clicked: bool,
    /// The mouse is hovering over the item
    pub hover: bool,
}

type ItemID = u64;

#[derive(Copy, Clone, Debug, Default)]
pub struct Layout {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Layout
{
    pub fn from_yoga_layout(parent: &Layout, layout: yoga::Layout) -> Layout {
        Layout {
            left: parent.left+layout.left(),
            top: parent.top+layout.top(),
            right: parent.left+layout.left()+layout.width(),
            bottom: parent.top+layout.top()+layout.height(),
        }
    }

    pub fn width(&self) -> f32 { self.right - self.left }
    pub fn height(&self) -> f32 { self.bottom - self.top }

    /*pub fn relative_to(&self, parent: &Layout) -> Layout {
        Layout {
            left: parent.left + self.left,
            top: parent.top + self.top,
            right: parent.left + self.right,
            bottom: parent.top + self.bottom,
        }
    }*/

    fn is_point_inside(&self, pos: (f32,f32)) -> bool {
        self.left <= pos.0 && pos.0 <= self.right && self.top <= pos.1 && pos.1 <= self.bottom
    }
}


/// Font description
#[derive(Clone, Debug)]
pub struct FontDesc(String);

/// Style
#[derive(Clone, Debug)]
pub enum Style
{
    FontFace(String),
    FontHeight(f32)
}

/// Style computed for an item
#[derive(Clone, Debug)]
pub struct ComputedStyle {
    font_face: String,
    font_height: f32,
}

/// Partially computed styles that may affect the measurements of the contents of an item.
#[derive(Clone)]
pub struct ContentMeasurementStyle {
    font_face: String,
    font_height: f32,
}

impl ContentMeasurementStyle
{
    pub fn add_style(&mut self, style: &Style) {
        match style {
            Style::FontFace(ref font) => self.font_face = font.clone(),
            Style::FontHeight(size) => self.font_height = *size,
        }
    }
}

impl Default for ContentMeasurementStyle
{
    fn default() -> ContentMeasurementStyle {
        ContentMeasurementStyle {
            font_face: "monospace".into(),
            font_height: 16.0
        }
    }
}

struct ContentMeasurement
{
    width: Option<f32>,
    height: Option<f32>,
}

/// A widget.
pub struct Item {
    flexbox: yoga::Node,
    prev_layout: Layout,
    children: IndexMap<ItemID, Item>,
    custom_data: Option<Box<Any>>,
    /// Non-layout styles associated to this widget
    styles: Vec<Style>,
    /// Optional callback for measuring the content
    measure_contents: Option<Box<Fn(&mut Item, &ContentMeasurementStyle, &Renderer)->ContentMeasurement>>,
    /// The draw callback, to draw stuff
    draw: Option<Box<Fn(&mut Item, &ComputedStyle, &Layout, &mut Renderer)>>,
    /// is the mouse hovering the element
    hovering: bool
}

/// Renderer & style interface
/// The UI passes computed styles and area information to the renderer for rendering.
/// The renderer gives required spacing and sizes of some elements.
/// Style information are CSS-like properties.
/// The renderer can do its own rendering with those styles (not obligated to follow them).
pub trait Renderer
{
    /// Measure the width of the given text under the given style.
    /// The full computed style must be available when measuring the text.
    /// This means that we need to compute the style inline during the UI update.
    /// This is not consistent with flexbox styles.
    fn measure_text(&self, text: &str, style: &ContentMeasurementStyle) -> f32;

    //fn draw_item(&mut self, item: &DrawItem, layout: &Layout, style: &ComputedStyle);
    fn draw_text(&mut self, text: &str, layout: &Layout, style: &ComputedStyle);
    fn draw_rect(&mut self, layout: &Layout, style: &ComputedStyle);
}

pub struct NvgRenderer<'ctx> {
    pub frame: nvg::Frame<'ctx>,
    pub default_font: nvg::Font<'ctx>,
    pub default_font_size: f32,
}

macro_rules! unwrap_enum {
    ($e:expr, ref mut $p:path) => {
        match $e { $p(ref mut e) => e, _ => panic!("unexpected enum variant") }
    };
    ($e:expr, ref $p:path) => {
        match $e { $p(ref e) => e, _ => panic!("unexpected enum variant") }
    };
    ($e:expr, $p:path) => {
        match $e { $p(e) => e, _ => panic!("unexpected enum variant") }
    };
}

impl<'ctx> Renderer for NvgRenderer<'ctx>
{
    fn measure_text(&self, text: &str, style: &ContentMeasurementStyle) -> f32 {
        let (advance, bounds) = self.frame.text_bounds(self.default_font, (0.0, 0.0), text, nvg::TextOptions {
            size: self.default_font_size,
            .. Default::default()
        });
        //debug!("text {} advance {}", text, advance);
        advance
    }

   /* fn draw_item(&mut self, item: &DrawItem, layout: &Layout, style: &ComputedStyle)
    {
        match item {
            &DrawItem::Text(ref text) => self.draw_text(text.as_ref(), layout, style),
            &DrawItem::Rect => self.draw_rect(layout, style),
            _ => {}
        }
    }*/

    fn draw_text(&mut self, text: &str, layout: &Layout, style: &ComputedStyle) {
        self.frame.text(
            self.default_font,
            (layout.left, layout.top),
            text,
            nvg::TextOptions {
                color: nvg::Color::new(1.0, 1.0, 1.0, 1.0),
                size: 14.0,
                ..Default::default()
            },
        );
    }

    fn draw_rect(&mut self, layout: &Layout, style: &ComputedStyle) {
        self.frame.path(
            |path| {
                path.rect((layout.left, layout.top), (layout.width(), layout.height()));
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
    }
}


// The flex layout may be influenced by some styles (notably, font & font-height)
// Cannot provide a preferred width during UI update.
// Must do a separate 'content measure' pass: consider only styles that may affect the measurements of the contents.
// Issue: cannot store the callback in the item because of double mut-borrow of the item
// callback: ref to custom data and

impl Item
{
    pub fn new() -> Item {
        Item {
            children: IndexMap::new(),
            flexbox: yoga::Node::new(),
            custom_data: None,
            prev_layout: Layout::default(),
            styles: Vec::new(),
            measure_contents: None,
            draw: None,
            hovering: false
        }
    }

    pub fn with_measure<F: Fn(&mut Item, &ContentMeasurementStyle, &Renderer)->ContentMeasurement + 'static>(&mut self, f: F) {
        self.measure_contents = Some(Box::new(f));
    }

    pub fn init_custom_data<D: Any>(&mut self, default: D) -> &mut D {
        if self.custom_data.is_none() {
            self.custom_data = Some(Box::new(default));
        }
        self.custom_data.as_mut().unwrap().downcast_mut().expect("wrong custom data type")
    }

    pub fn get_custom_data<D: Any>(&self) -> &D {
        self.custom_data.as_ref().unwrap().downcast_ref::<D>().unwrap()
    }

    pub fn get_custom_data_mut<D: Any>(&mut self) -> &mut D {
        self.custom_data.as_mut().unwrap().downcast_mut::<D>().unwrap()
    }

    pub fn apply_styles<'b, I>(&mut self, styles: I)
        where I: IntoIterator<Item = &'b yoga::FlexStyle>
    {
        self.flexbox.apply_styles(styles);
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
    root: Item,
    state: UiState,
}

struct FilteredEvent
{
    captured: bool,
    event: WindowEvent,
}

// style interface:
// - measure text
// - measure other elements susceptible of having a preferred size depending on the style
// -> XXX: is it good that some elements size depend on the style? can affect the layout.
//         CSS does

//
// Styles: vec of Style enum

pub struct UiState
{
    id_stack: IdStack,
    events: Vec<FilteredEvent>,
    cur_frame: u64,
    last_input_state: InputState,
}

impl UiState
{
    pub fn new() -> UiState {
        UiState {
            id_stack: IdStack::new(0),
            cur_frame: 0,
            events: Vec::new(),
            last_input_state: InputState {
                cursor_pos: (0.0,0.0)
            }
        }
    }

    pub fn measure_item(&mut self, id: ItemID, item: &mut Item, renderer: &Renderer, parent_content_style: &ContentMeasurementStyle)
    {
        let mut style = parent_content_style.clone();
        for s in item.styles.iter() {
            style.add_style(s);
        }

        // move the closure outside the item so we don't hold a borrow
        let measure = replace(&mut item.measure_contents, None);

        if let Some(ref measure) = measure {
            //
            let m = measure(item, &style, renderer);
            if let Some(width) = m.width {
                style!(item.flexbox, Width(width.point()))
            }
            if let Some(height) = m.height {
                style!(item.flexbox, Height(height.point()))
            }
        }

        // move the closure back inside
        replace(&mut item.measure_contents, measure);

        for (&id,child) in item.children.iter_mut() {
            self.measure_item(id, child, renderer, &style);
        }
    }


    pub fn render_item(&mut self, id: ItemID, item: &mut Item, parent_layout: &Layout, renderer: &mut Renderer)
    {
        let flex_layout = item.flexbox.get_layout();
        let layout = Layout::from_yoga_layout(parent_layout, item.flexbox.get_layout());
        // TODO not dummy
        let computed_styles = ComputedStyle {
            font_face: "monospace".into(),
            font_height: 16.0
        };

        item.prev_layout = layout;

        // move the closure outside the item so we don't hold a borrow
        // move the closure back inside afterwards
        let draw = replace(&mut item.draw, None);
        if let Some(ref draw) = draw {
            draw(item, &computed_styles, &layout, renderer);
        }
        replace(&mut item.draw, draw);

        //renderer.draw_item(&item.draw_item, &layout, &computed_styles);
        //renderer.draw_rect(&layout, &computed_styles);
        //renderer.draw_text(&format!("{:016X}", id), &layout, &computed_styles);

        for (&id,child) in item.children.iter_mut() {
            self.render_item(id, child, &layout, renderer);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Padding {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

fn measure_time<F: FnOnce()>(f: F) -> u64 {
    let start = time::PreciseTime::now();
    f();
    let duration = start.to(time::PreciseTime::now());
    duration.num_microseconds().unwrap() as u64
}

impl Ui {
    pub fn new() -> Ui {
        let mut root = Item::new();

        let mut ui = Ui {
            root,
            state: UiState::new()
        };
        ui
    }

    pub fn event(&mut self, event: &WindowEvent) {
        self.state.events.push(FilteredEvent {
            captured: false,
            event: event.clone()
        });
    }

    pub fn root<F: FnOnce(&mut UiContainer)>(&mut self, f: F)
    {
        let spec_time = measure_time(|| {
            let root = &mut self.root;
            let mut container = UiContainer::new(0, root, &mut self.state);
            f(&mut container);
        });
        debug!("ui specification took {}us", spec_time);
    }

    pub fn render(&mut self, size: (f32, f32), renderer: &mut Renderer)
    {
        self.state.events.clear();
        // measure contents pass
        let content_style = ContentMeasurementStyle::default();
        let measure_contents_time = measure_time(|| {
            self.state.measure_item(0, &mut self.root, renderer, &content_style);
        });
        let layout_time = measure_time(|| {
            self.root.flexbox.calculate_layout(size.0, size.1, yoga::Direction::LTR);
        });
        let root_layout = Layout {
            left: 0.0,
            top: 0.0,
            right: size.0,
            bottom: size.1,
        };
        let render_time = measure_time(|| {
            self.state.render_item(0, &mut self.root, &root_layout, renderer);
        });

        debug!("measure {}us, layout {}us, render {}us", measure_contents_time, layout_time, render_time);
    }
}

pub struct UiContainer<'a> {
    state: &'a mut UiState,
    pub id: ItemID,
    pub item: &'a mut Item,
    cur_index: usize
}

// Input is stored in a list of events, then distributed to the items
// each item can consume one or more events from the list

impl<'a> UiContainer<'a>
{
    pub fn new_item<'s, F: FnOnce()->Item>(&'s mut self, new_item_id: ItemID, f: F) -> UiContainer<'s> {
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
                Entry::Occupied(mut entry) => {
                    let index = entry.index();
                    // if the child item exists, see if its index corresponds
                    if cur_index != index {
                        // item has moved: extract the item from its previous position
                        self.item.flexbox.remove_child(&mut entry.get_mut().flexbox);
                        Some(entry.remove())
                    } else {
                        // child item exists and has not moved
                        None
                    }
                }
            }
            // drop borrow by entry()
        };

        if let Some(mut item) = item_reinsert {
            // must insert or reinsert an item at the correct index
            // insert last
            self.item.flexbox.insert_child(&mut item.flexbox, cur_index as u32);
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
            state: self.state,
            item: new_item,
            id: new_item_id,
            cur_index: 0
        }
    }

    pub fn new(id: ItemID, item: &'a mut Item, state: &'a mut UiState) -> UiContainer<'a> {
        UiContainer {
            state,
            item,
            id,
            cur_index: 0
        }
    }
}


// so, event filtering is complicated, because we have a sequence of events
// that may modify some variables, such as the cursor position.
// i.e. the cursor position changes within the event stream.
// The event filter must keep track of such changes to the cursor position to have a correct hit-test.
// This also means that there are some events that we cannot remove from the stream.
// Actually, never remove any event: just set a flag that indicates whether the event has been processed.
// API?
// filter_events also handles the update of the item state (Hovered, Focused, etc.)

pub struct InputState {
    cursor_pos: (f32,f32),
}


impl<'a> UiContainer<'a>
{
    pub fn filter_events<F: FnMut(&mut Item, &WindowEvent, &InputState) -> bool>(&mut self, mut f: F)
    {
        // filter events that pass the hit test
        let id = self.id;
        let item = &mut *self.item;
        let input_state = &mut self.state.last_input_state;

        self.state.events.iter_mut().filter(|event| !event.captured).for_each(|filtered_event| {
            // filter events and update input state
            let pass = match &filtered_event.event {
                // TODO check if item has focus
                &WindowEvent::KeyboardInput { device_id, input } => true,
                &WindowEvent::CursorMoved {
                    device_id,
                    position,
                    modifiers
                } => {
                    let position = (position.0 as f32, position.1 as f32);
                    input_state.cursor_pos = position;
                    item.prev_layout.is_point_inside(position)
                },
                &WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                    item.prev_layout.is_point_inside(input_state.cursor_pos)
                },
                // TODO
                _ => true
            };

            if !pass {
                // event not for us, pass it along (capture or bubble up)
                return
            }

            debug!("HIT PASS {:016X}", id);

            // event for us, process it
            let captured = f(item, &filtered_event.event, &input_state);
            filtered_event.captured = captured;
        });
    }


    /// Set the draw callback, **if** it has not already been set.
    pub fn draw<F: Fn(&mut Item, &ComputedStyle, &Layout, &mut Renderer) + 'static>(&mut self, f: F) {
        self.item.draw.get_or_insert_with(|| { Box::new(f) });
    }

    pub fn with_id<S: Into<String>, F: FnOnce(&mut UiContainer, ItemID)>(
        &mut self,
        id: S,
        f: F)
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.state.id_stack.push_id(&id_str);
        f(self, id);
        self.state.id_stack.pop_id();
    }

    pub fn item<S: Into<String>, F: FnOnce(&mut UiContainer)>(
        &mut self,
        id: S,
        f: F)
    {
        // convert ID to string for later storage
        let id_str = id.into();
        // get numeric ID
        let id = self.state.id_stack.push_id(&id_str);
        {
            let mut container = self.new_item(id, || { Item::new() });
            /*style!(container.item,
                FlexDirection(yoga::FlexDirection::Column)
                //FlexGrow(1.0)
            );*/
            f(&mut container);
        }
        self.state.id_stack.pop_id();
    }


    pub fn vbox<S: Into<String>, F: FnOnce(&mut UiContainer)>(
        &mut self,
        id: S,
        f: F,
    ) -> ItemResult
    {
        self.item(id, |ui| {
            ui.draw(|item,style,layout,renderer| {
                renderer.draw_rect(layout, style);
            });

            style!(ui.item,
                FlexDirection(yoga::FlexDirection::Column),
                FlexGrow(1.0),
                Margin(2.0 pt)
            );
            f(ui);

            let itemid = ui.id;
            ui.filter_events(|item,event,input_state| {
                debug!("Item {:016X} received event {:?} cursor_pos={:?}", itemid, event, input_state.cursor_pos);
                true    // do not bubble
            });
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    pub fn text<S: Into<String>+Clone>(
        &mut self,
        text: S
    ) -> ItemResult
    {
        self.item(text.clone(), |ui| {
            ui.item.init_custom_data(text.into());

            ui.draw(|item,style,layout,renderer| {
                let s = item.get_custom_data::<String>();
                renderer.draw_text(s, layout, style);
            });

            ui.item.with_measure(|item,style,renderer| {
                let s = item.get_custom_data::<String>();
                let m = renderer.measure_text(s.as_ref(), style);
                ContentMeasurement {
                    width: Some(m),
                    height: Some(style.font_height),
                }
            });
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    /// a scrollable panel
    pub fn scroll<S: Into<String>, F: FnOnce(&mut UiContainer)>(
        &mut self,
        id: S,
        f: F,
    )
    {
        struct ScrollState {
            pos: f32
        };

        self.item(id, |ui| {
            ui.item.init_custom_data(ScrollState { pos: 0.0 });
            ui.filter_events(|item,event,input_state| {
                let state = item.get_custom_data_mut::<ScrollState>();
                match event {
                    &WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Up) => {debug!("Scroll up"); state.pos -= 10.0;}
                            Some(VirtualKeyCode::Down) => {debug!("Scroll down"); state.pos += 10.0;}
                            _ => {}
                        }
                    },
                    _ => {}
                }
                // always capture?
                false
            });

            let top = -ui.item.get_custom_data::<ScrollState>().pos;

            style!(ui.item,
                FlexDirection(yoga::FlexDirection::Column),
                FlexGrow(1.0),
                Margin(4.0 pt),
                Top(top.point())
            );



            ui.draw(|item,style,layout,renderer| {
                renderer.draw_rect(layout, style);
            });

            f(ui);
        });
    }
}

// Item:
// - layouter (in function)
// - draw element (Boxed)

// Creating a custom control with state:
// ui.item(id, ...).with_data(...).layout(...)
// ui.item() -> UiContainer
// ui.item(id, |ui| {
//    let mut item = ui.item;
//    let mut custom_data = ui.with_data(default);
//    <do something with custom data>
//    f(ui);
//    <layout>
// })
