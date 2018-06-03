use super::layout::{Layout};
use super::renderer::{DrawItem, DrawItemKind, Renderer};
use super::style::{CachedStyle};

use nvg;

pub struct NvgRenderer<'ctx> {
    frame: nvg::Frame<'ctx>,
    default_font: nvg::Font<'ctx>,
    default_font_size: f32,
}

impl<'ctx> NvgRenderer<'ctx> {
    pub fn new(
        frame: nvg::Frame<'ctx>,
        default_font: nvg::Font<'ctx>,
        default_font_size: f32,
        //image_cache: &'cache ImageCache<'ctx>,
    ) -> NvgRenderer<'ctx> {
        NvgRenderer {
            frame,
            default_font,
            default_font_size,
            //image_cache,
        }
    }
}

impl<'ctx> NvgRenderer<'ctx> {
    fn draw_text(&mut self, text: &str, layout: &Layout, style: &CachedStyle) {
        self.frame.text(
            self.default_font,
            (layout.left, layout.top),
            text,
            nvg::TextOptions {
                color: nvg::Color::new(1.0, 1.0, 1.0, 1.0),
                size: 16.0,
                ..Default::default()
            },
        );
    }

    fn draw_rect(&mut self, layout: &Layout, style: &CachedStyle) {
        // convert style to nvg styles
        let fill_paint = {
            let (r, g, b, a) = style.non_layout.background_color;
            nvg::Color::new(r, g, b, a)
        };

        let stroke_paint = {
            let (r, g, b, a) = style.non_layout.border_color.top;
            nvg::Color::new(r, g, b, a)
        };

        let border_width = style.non_layout.border_width.top;
        let stroke_opts = nvg::StrokeOptions {
            width: border_width,
            ..Default::default()
        };
        let border_radius = style.non_layout.border_radius;

        //debug!("draw layout: {:?}", layout);

        // Fill path
        self.frame.path(
            |path| {
                if border_radius != 0.0 {
                    path.rounded_rect(
                        (layout.left, layout.top),
                        (layout.width(), layout.height()),
                        border_radius,
                    );
                } else {
                    path.rect((layout.left, layout.top), (layout.width(), layout.height()));
                }
                path.fill(fill_paint, Default::default());
            },
            Default::default(),
        );

        self.frame.path(
            |path| {
                if border_radius != 0.0 {
                    path.rounded_rect(
                        (layout.left, layout.top),
                        (layout.width(), layout.height()),
                        border_radius,
                    );
                } else {
                    path.rect(
                        (
                            layout.left + 0.5 * border_width,
                            layout.top + 0.5 * border_width,
                        ),
                        (
                            layout.width() - border_width,
                            layout.height() - border_width,
                        ),
                    );
                }
                path.stroke(stroke_paint, stroke_opts);
            },
            Default::default(),
        );
    }

    fn draw_image(&mut self, image_path: &str, layout: &Layout) {
        /*let img = self.image_cache.get_or_load_image(image_path);
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
        });*/
    }
}

impl<'ctx> Renderer for NvgRenderer<'ctx> {
    fn measure_text(&self, text: &str, _style: &CachedStyle) -> f32 {
        let (advance, _bounds) = self.frame.text_bounds(
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
        /*let img = self.image_cache.get_or_load_image(img);
        img.map(|img| {
            let (w, h) = img.size();
            (w as f32, h as f32)
        })*/
        unimplemented!()
    }

    fn draw_frame(&mut self, items: &[DrawItem]) {
        for di in items {
            match di.kind {
                DrawItemKind::Rect => {
                    self.draw_rect(&di.layout, &di.style);
                }
                DrawItemKind::Image(_) => unimplemented!(),
                DrawItemKind::Text(ref str) => {
                    self.draw_text(str, &di.layout, &di.style);
                }
            }
        }
    }
}
