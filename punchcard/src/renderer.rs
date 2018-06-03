use super::layout::Layout;
use super::style::{
    Background, CachedStyle, Color, ComputedStyle, GradientStop, LinearGradient, RadialGradient,
};
use super::ResourceStore;

use std::cell::RefCell;
use std::collections::{
    hash_map::{Entry, OccupiedEntry, VacantEntry}, HashMap,
};
use std::rc::Rc;

pub enum DrawItemKind {
    Text(String),
    Image(String),
    Rect,
}

/// Item of a draw list.
pub struct DrawItem {
    pub z_order: u32,
    pub style: CachedStyle,
    pub layout: Layout,
    pub kind: DrawItemKind,
}

impl DrawItem {
    pub(super) fn new_rect(layout: Layout, style: CachedStyle, z_order: Option<u32>) -> DrawItem {
        DrawItem {
            z_order: z_order.unwrap_or(0),
            style,
            layout,
            kind: DrawItemKind::Rect,
        }
    }

    pub(super) fn new_text(
        text: String,
        layout: Layout,
        style: CachedStyle,
        z_order: Option<u32>,
    ) -> DrawItem {
        DrawItem {
            z_order: z_order.unwrap_or(0),
            style,
            layout,
            kind: DrawItemKind::Text(text),
        }
    }

    pub(super) fn new_image(
        text: String,
        layout: Layout,
        style: CachedStyle,
        z_order: Option<u32>,
    ) -> DrawItem {
        unimplemented!()
    }
}

pub struct DrawList {
    pub(super) items: Vec<DrawItem>,
}

impl DrawList {
    pub(super) fn new() -> DrawList {
        DrawList { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: DrawItem) {
        self.items.push(item);
    }

    pub fn add_rect(&mut self, layout: Layout, style: CachedStyle, z_order: Option<u32>) {
        self.items.push(DrawItem::new_rect(layout, style, z_order));
    }

    pub fn add_text(
        &mut self,
        text: String,
        layout: Layout,
        style: CachedStyle,
        z_order: Option<u32>,
    ) {
        self.items
            .push(DrawItem::new_text(text, layout, style, z_order));
    }

    pub(super) fn sort(&mut self) {
        self.items.sort_by(|a, b| a.z_order.cmp(&b.z_order));
    }
}

/// Renderer & style interface
/// The UI passes computed styles and area information to the renderer for rendering.
/// The renderer gives required spacing and sizes of some elements.
/// Style information are CSS-like properties.
/// The renderer can do its own rendering with those styles (not obligated to follow them).
///
pub trait Renderer {
    /// Measures the width of the given text under the given style.
    /// The full computed style must be available when measuring the text.
    /// This means that we need to compute the style inline during the UI update.
    /// This is not consistent with flexbox styles.
    fn measure_text(&self, text: &str, style: &CachedStyle) -> f32;
    /// Measures the dimensions of the image at the given path.
    fn measure_image(&self, image_path: &str) -> Option<(f32, f32)>;

    /// Draws the draw list.
    /// The draw list is already correctly sorted by z-order.
    fn draw_frame(&mut self, items: &[DrawItem]);
}
