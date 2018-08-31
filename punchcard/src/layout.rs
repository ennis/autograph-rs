use yoga;

#[derive(Copy, Clone, Debug, Default)]
pub struct Bounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Bounds {
    pub fn from_yoga_layout(parent: &Bounds, layout: yoga::Layout) -> Bounds {
        Bounds {
            left: parent.left + layout.left(),
            top: parent.top + layout.top(),
            right: parent.left + layout.left() + layout.width(),
            bottom: parent.top + layout.top() + layout.height(),
        }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn is_point_inside(&self, pos: (f32, f32)) -> bool {
        self.left <= pos.0 && pos.0 <= self.right && self.top <= pos.1 && pos.1 <= self.bottom
    }

    pub fn relative_position(&self, pos: (f32, f32)) -> (f32, f32) {
        (pos.0 - self.left, pos.1 - self.top)
    }
}

pub struct ContentMeasurement {
    pub width: Option<f32>,
    pub height: Option<f32>,
}
