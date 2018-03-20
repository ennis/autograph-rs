use nvg;

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

pub struct Ui
{
    draw_items: Vec<DrawItem>
}

// NONE of the methods should return a borrowed value (this will block everything)
// Of course, UI submission should be single-threaded
// layouts? nested things?
// begin/end pairs? closures?
impl Ui
{
    pub fn new() -> Ui {
        Ui {
            draw_items: Vec::new()
        }
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