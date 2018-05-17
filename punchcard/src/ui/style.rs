/// Font description
#[derive(Clone, Debug)]
pub struct FontDesc(String);

pub type Color = (f32, f32, f32, f32);

/// Style.
#[derive(Clone, Debug)]
pub enum StyleRule {
    FontFace(String),
    FontHeight(f32),
    Background(Background),
    BackgroundColor(Color),
    BorderColor(Color),
    BorderRadius(Color),
}

/// Border style
#[derive(Copy, Clone, Debug)]
pub enum BorderStyle
{
    Default
}

/// Gradient stop.
#[derive(Clone, Debug)]
pub struct GradientStop {
    color: Color,
}

/// Linear gradient.
#[derive(Clone, Debug)]
pub struct LinearGradient {
    angle: f32,
    stops: Vec<GradientStop>,
}

/// Radial gradient.
#[derive(Clone, Debug)]
pub struct RadialGradient {
    stops: Vec<GradientStop>,
}

/// Background style.
#[derive(Clone, Debug)]
pub enum Background {
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

/// Visual style
#[derive(Clone, Debug)]
pub struct Style {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_bottom_color: Option<Color>,
    pub border_left_color: Option<Color>,
    pub border_right_color: Option<Color>,
    pub border_top_color: Option<Color>,
    pub border_bottom_width: Option<f32>,
    pub border_left_width: Option<f32>,
    pub border_right_width: Option<f32>,
    pub border_top_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub background: Option<Background>,
}

macro_rules! inherit_props {
    ($left:expr, $parent:expr, $($prop:ident),*) => {
        Style {
            $($prop: $left.$prop.clone().or($parent.$prop.clone()),)*
            .. $left.clone()
        }
    };
}

impl Style {
    /// Returns the empty style.
    pub fn empty() -> Style {
        Style {
            font_family: None,
            font_size: None,
            font_color: None,
            background_color: None,
            border_bottom_color: None,
            border_left_color: None,
            border_right_color: None,
            border_top_color: None,
            border_bottom_width: None,
            border_left_width: None,
            border_right_width: None,
            border_top_width: None,
            border_radius: None,
            background: None,
        }
    }

    /// Computes inherited properties for this style.
    pub fn inherit(&self, parent: &Style) -> Style {
        inherit_props!(self, parent, font_family, font_size, font_color)
    }

    /// Sets all undefined styles to the default values provided.
    pub fn with_default(&self, parent: &Style) -> Style {
        inherit_props!(
            self,
            parent,
            font_family,
            font_size,
            font_color,
            background_color,
            border_bottom_color,
            border_left_color,
            border_right_color,
            border_top_color,
            border_radius,
            border_bottom_width,
            border_left_width,
            border_right_width,
            border_top_width,
            background
        )
    }

    /// Sets the border color.
    pub fn set_border_color(&mut self, color: Color) {
        self.border_bottom_color = Some(color);
        self.border_left_color = Some(color);
        self.border_right_color = Some(color);
        self.border_top_color = Some(color);
    }

    /// Sets the width of the border.
    pub fn set_border_width(&mut self, width: f32) {
        self.border_bottom_width = Some(width);
        self.border_left_width = Some(width);
        self.border_right_width = Some(width);
        self.border_top_width = Some(width);
    }

    /// Sets the border radius.
    pub fn set_border_radius(&mut self, radius: f32) {
        self.border_radius = Some(radius);
    }

    /// Sets the background color.
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = Some(color);
    }
}
