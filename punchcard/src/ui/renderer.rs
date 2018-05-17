use super::layout::Layout;
use super::style::{Background, Color, GradientStop, LinearGradient, RadialGradient, Style};

use std::cell::RefCell;
use std::collections::{hash_map::{Entry, OccupiedEntry, VacantEntry},
                       HashMap};
use std::rc::Rc;

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
    /// Measure the dimensions of the image at the given path.
    fn measure_image(&self, image_path: &str) -> Option<(f32, f32)>;

    //fn draw_item(&mut self, item: &DrawItem, layout: &Layout, style: &ComputedStyle);
    fn draw_text(&mut self, text: &str, layout: &Layout, style: &Style);
    fn draw_rect(&mut self, layout: &Layout, style: &Style);
    fn draw_image(&mut self, image_path: &str, layout: &Layout);
}

pub struct ImageCache<'ctx> {
    context: &'ctx nvg::Context,
    // TODO support hot-reload of images
    cache: RefCell<HashMap<String, Rc<nvg::Image<'ctx>>>>,
}

impl<'ctx> ImageCache<'ctx> {
    pub fn new(context: &'ctx nvg::Context) -> ImageCache<'ctx> {
        ImageCache {
            context,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_or_load_image(&self, image_path: &str) -> Option<Rc<nvg::Image<'ctx>>> {
        //unimplemented!()
        let mut cache = self.cache.borrow_mut();
        let entry = cache.entry(image_path.to_string());
        match entry {
            Entry::Occupied(entry) => Some(entry.get().clone()),
            Entry::Vacant(entry) => {
                let img = nvg::Image::new(self.context).build_from_file(image_path);
                match img {
                    Ok(img) => Some(entry.insert(Rc::new(img)).clone()),
                    Err(err) => {
                        error!("Failed to load image `{}'", image_path);
                        None
                    }
                }
            }
        }
    }
}

pub struct NvgRenderer<'cache, 'ctx: 'cache> {
    frame: nvg::Frame<'ctx>,
    default_font: nvg::Font<'ctx>,
    default_font_size: f32,
    image_cache: &'cache ImageCache<'ctx>,
}

impl<'cache, 'ctx: 'cache> NvgRenderer<'cache, 'ctx> {
    pub fn new(
        frame: nvg::Frame<'ctx>,
        default_font: nvg::Font<'ctx>,
        default_font_size: f32,
        image_cache: &'cache ImageCache<'ctx>,
    ) -> NvgRenderer<'cache, 'ctx> {
        NvgRenderer {
            frame,
            default_font,
            default_font_size,
            image_cache,
        }
    }
}

impl<'cache, 'ctx: 'cache> Renderer for NvgRenderer<'cache, 'ctx> {
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

    fn measure_image(&self, img: &str) -> Option<(f32, f32)> {
        let img = self.image_cache.get_or_load_image(img);
        img.map(|img| {
            let (w, h) = img.size();
            (w as f32, h as f32)
        })
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
        // convert style to nvg styles


        let fill_paint = match style.background {
                Some(Background::RadialGradient(ref gradient)) => unimplemented!(),
                Some(Background::LinearGradient(ref gradient)) => unimplemented!(),
                None => match style.background_color {
                    Some((r, g, b, a)) => nvg::Color::new(r, g, b, a),
                    None => {
                        warn!("empty style property: background color");
                        nvg::Color::new(0.0, 0.0, 0.0, 0.0)
                    }
                },
        	};

        let stroke_paint = match style.border_top_color {
                Some((r, g, b, a)) => nvg::Color::new(r, g, b, a),
                None => {
                    warn!("empty style property: border color");
                    nvg::Color::new(0.0, 0.0, 0.0, 0.0)
                }
            };

        self.frame.path(
            |path| {
                if let Some(border_radius) = style.border_radius {
                    path.rounded_rect(
                        (layout.left, layout.top),
                        (layout.width(), layout.height()),
                        border_radius,
                    );
                } else {
                    path.rect((layout.left, layout.top), (layout.width(), layout.height()));
                }
                path.stroke(stroke_paint, Default::default());
                path.fill(fill_paint, Default::default());
            },
            Default::default(),
        );
    }

    fn draw_image(&mut self, image_path: &str, layout: &Layout) {
        let img = self.image_cache.get_or_load_image(image_path);
        img.map(|img| {
            self.frame.path(
                |path| {
                    let (w, h) = (layout.width(), layout.height());
                    path.rect((layout.left, layout.top), (layout.width(), layout.height()));
                    let pattern = nvg::ImagePattern {
                        image: img.as_ref(),
                        origin: (0.0, 0.0),
                        size: (w, h),
                        angle: 0.0,
                        alpha: 1.0,
                    };
                    path.fill(pattern, Default::default());
                },
                Default::default(),
            );
        });
    }
}
