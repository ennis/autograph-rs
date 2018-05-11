/// Font description
#[derive(Clone, Debug)]
pub struct FontDesc(String);

pub type Color = (f32, f32, f32, f32);

/// Style.
#[derive(Clone, Debug)]
pub enum StyleE {
    FontFace(String),
    FontHeight(f32),
    BackgroundColor(Color),
    BorderColor(Color),
    BorderRadius(Color),
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
    pub border_color: Option<Color>,
    pub border_radius: Option<f32>,
    pub background: Option<Background>,
}

impl Style {
    /// Return an empty style.
    pub fn empty() -> Style {
        Style {
            font_family: None,
            font_size: None,
            font_color: None,
            background_color: None,
            border_color: None,
            border_radius: None,
            background: None,
        }
    }

    /// Compute inherited properties for this style.
    pub fn inherit(&self, parent: &Style) -> Style {
        // inherited properties:
        // font-family
        // font-size
        // font-color
        Style {
            font_family: self.font_family.clone().or(parent.font_family.clone()),
            font_size: self.font_size.clone().or(parent.font_size.clone()),
            font_color: self.font_color.clone().or(parent.font_color.clone()),
            ..self.clone()
        }
    }

    /// Set all undefined styles to the default values provided.
    pub fn with_default(&self, parent: &Style) -> Style {
        Style {
            font_family: self.font_family.clone().or(parent.font_family.clone()),
            font_size: self.font_size.clone().or(parent.font_size.clone()),
            font_color: self.font_color.clone().or(parent.font_color.clone()),
            background_color: self.background_color
                .clone()
                .or(parent.background_color.clone()),
            border_color: self.border_color.clone().or(parent.border_color.clone()),
            border_radius: self.border_radius.clone().or(parent.border_radius.clone()),
            background: self.background.clone().or(parent.background.clone()),
        }
    }
}
