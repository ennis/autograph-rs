use super::layout::{Layout};
use super::{Arena, NodeId, RetainedNode};
use super::style::*;

use glutin::{GlWindow,EventsLoop};
use nanovg as nvg;

pub struct Renderer {
    context: nvg::Context,
    default_font_name: String,
    default_font_size: f32,
}

const DEFAULT_FONT: &'static str = "data/fonts/iosevka-regular.ttf";

pub(super) fn render_node<'ctx>(frame: &nvg::Frame<'ctx>, arena: &mut Arena<RetainedNode>, id: NodeId, parent_layout: &Layout)
{
    let layout = {
        let node = &mut arena[id];
        let data = node.data_mut();
        let layout = data.update_layout(parent_layout);

        match data.contents {
            Contents::Text(ref text) => {
                // TODO
            },
            Contents::Element => {
                render_rect(frame, id, &layout, data.styles.as_ref().expect("styles were not computed before render"));
            }
        };
        // return layout, drop borrow of arena
        layout
    };

    let mut next = arena[id].first_child();
    while let Some(id) = next {
        render_node(frame, arena, id, &layout);
        next = arena[id].next_sibling();
    }
}


impl Renderer {
    pub fn new(main_window: &glutin::Window, events_loop: &glutin::EventsLoop) -> Renderer
    {
        let context = nvg::ContextBuilder::new()
            .stencil_strokes()
            .build()
            .expect("Initialization of NanoVG failed!");

        let iosevka_font = nvg::Font::from_file(
            &context,
            "default",
            DEFAULT_FONT,
        ).expect("Failed to load default font");

        Renderer {
            context,
            default_font_name: "default".to_owned(),
            default_font_size: 14.0,
            //image_cache,
        }
    }



    pub fn layout_and_render_dom(&mut self, window: &glutin::Window, arena: &mut Arena<RetainedNode>, root: NodeId)
    {

    }
}

impl Renderer {
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

impl<'ctx> Renderer<'ctx> {
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
