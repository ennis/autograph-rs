use super::layout::Layout;
use super::style::Style;

use nvg;

/// Renderer & style interface
/// The UI passes computed styles and area information to the renderer for rendering.
/// The renderer gives required spacing and sizes of some elements.
/// Style information are CSS-like properties.
/// The renderer can do its own rendering with those styles (not obligated to follow them).
pub trait Renderer {
    /// Measure the width of the given text under the given style.
    /// The full computed style must be available when measuring the text.
    /// This means that we need to compute the style inline during the UI update.
    /// This is not consistent with flexbox styles.
    fn measure_text(&self, text: &str, style: &Style) -> f32;

    //fn draw_item(&mut self, item: &DrawItem, layout: &Layout, style: &ComputedStyle);
    fn draw_text(&mut self, text: &str, layout: &Layout, style: &Style);
    fn draw_rect(&mut self, layout: &Layout, style: &Style);
}

pub struct NvgRenderer<'ctx> {
    pub frame: nvg::Frame<'ctx>,
    pub default_font: nvg::Font<'ctx>,
    pub default_font_size: f32,
}

impl<'ctx> Renderer for NvgRenderer<'ctx> {
    fn measure_text(&self, text: &str, style: &Style) -> f32 {
        let (advance, bounds) = self.frame.text_bounds(
            self.default_font,
            (0.0, 0.0),
            text,
            nvg::TextOptions {
                size: self.default_font_size,
                ..Default::default()
            },
        );
        //debug!("text {} advance {}", text, advance);
        advance
    }

    fn draw_text(&mut self, text: &str, layout: &Layout, style: &Style) {
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

    fn draw_rect(&mut self, layout: &Layout, style: &Style) {
        self.frame.path(
            |path| {
                path.rect((layout.left, layout.top), (layout.width(), layout.height()));
                path.stroke(nvg::StrokeStyle {
                    coloring_style: nvg::ColoringStyle::Color(nvg::Color::new(0.5, 0.5, 0.5, 1.0)),
                    width: 1.0,
                    ..Default::default()
                });
            },
            Default::default(),
        );
    }
}
