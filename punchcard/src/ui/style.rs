use super::css;
use yoga;
use yoga::prelude::*;
use std::rc::Rc;

// Style system V2
// Split computed style in independent shareable style blocks.
// Borders, Background fill, Font (all inheritable), Layout properties, dynamic layout properties (left, right, etc.) that do not required full relayout.
//
// Issue: layout prop removed in inline CSS
// - recalc
// - update


/// Font description
#[derive(Clone, Debug)]
pub struct FontDesc(String);

pub type Color = (f32, f32, f32, f32);

/// Calculated border and background style.
/// Changing these should not trigger a full relayout.
#[derive(Clone, Debug, PartialEq)]
pub struct NonLayoutStyles
{
    pub background_color: Color,
    pub border_color: BoxProperty<Color>,
    pub border_width: BoxProperty<f32>,
    pub border_radius: f32,
}

impl Default for NonLayoutStyles
{
    fn default() -> NonLayoutStyles
    {
        NonLayoutStyles {
            background_color: (0.0, 0.0, 0.0, 0.0),
            border_color: BoxProperty::all((0.0,0.0,0.0,0.0)),
            border_width: BoxProperty::all(0.0),
            border_radius: 0.0,
        }
    }
}

/// Calculated layout properties.
/// These properties can trigger a relayout when changed and are not meant to change frequently.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutStyles
{
    pub display: yoga::Display,
    pub align_content: yoga::Align,
    pub align_items: yoga::Align,
    pub align_self: yoga::Align,
    pub aspect_ratio: f32,
    pub position: yoga::PositionType,
    pub flex_basis: yoga::StyleUnit,
    pub flex_direction: yoga::FlexDirection,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_wrap: yoga::Wrap,
    pub justify_content: yoga::Justify,
    pub margin: BoxProperty<yoga::StyleUnit>,
    pub max_height: yoga::StyleUnit,
    pub max_width: yoga::StyleUnit,
    pub min_height: yoga::StyleUnit,
    pub min_width: yoga::StyleUnit,
    pub overflow: yoga::Overflow,
    pub padding: BoxProperty<yoga::StyleUnit>,
}

impl Default for LayoutStyles
{
    fn default() -> LayoutStyles {
        LayoutStyles {
            display: yoga::Display::Flex,
            align_content: yoga::Align::Auto,
            align_items: yoga::Align::Auto,
            align_self: yoga::Align::Auto,
            aspect_ratio: yoga::Undefined,
            position: yoga::PositionType::Relative,
            flex_basis: yoga::StyleUnit::UndefinedValue,
            flex_direction: yoga::FlexDirection::Column,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_wrap: yoga::Wrap::NoWrap,
            justify_content: yoga::Justify::FlexStart,
            margin: BoxProperty::all(yoga::StyleUnit::UndefinedValue),
            max_height: yoga::StyleUnit::UndefinedValue,
            max_width: yoga::StyleUnit::UndefinedValue,
            min_height: yoga::StyleUnit::UndefinedValue,
            min_width: yoga::StyleUnit::UndefinedValue,
            overflow: yoga::Overflow::Visible,
            padding: BoxProperty::all(yoga::StyleUnit::UndefinedValue),
        }
    }
}


/// Dynamic layout properties.
/// These are expected to change more frequently than the other layout properties,
/// but they do not trigger a full relayout.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DynamicLayoutStyles
{
    pub left: yoga::StyleUnit,
    pub right: yoga::StyleUnit,
    pub top: yoga::StyleUnit,
    pub bottom: yoga::StyleUnit,
    pub width: yoga::StyleUnit,
    pub height: yoga::StyleUnit,
}

impl Default for DynamicLayoutStyles
{
    fn default() -> DynamicLayoutStyles
    {
        DynamicLayoutStyles {
            left: yoga::StyleUnit::UndefinedValue,
            right: yoga::StyleUnit::UndefinedValue,
            top: yoga::StyleUnit::UndefinedValue,
            bottom: yoga::StyleUnit::UndefinedValue,
            width: yoga::StyleUnit::UndefinedValue,
            height: yoga::StyleUnit::UndefinedValue,
        }
    }
}

pub struct ComputedStyle2
{
    pub non_layout: NonLayoutStyles,
    pub layout: LayoutStyles,
    pub dyn_layout: DynamicLayoutStyles,
}

impl Default for ComputedStyle2
{
    fn default() -> ComputedStyle2 {
        ComputedStyle2 {
            non_layout: Default::default(),
            layout: Default::default(),
            dyn_layout: Default::default()
        }
    }
}

impl ComputedStyle2
{
    /// Apply CSS property.
    pub(super) fn apply_property(&mut self, prop: &css::PropertyDeclaration) {
        match prop {
            // Non-layout styles
            css::PropertyDeclaration::Color(c) => { unimplemented!() },
            css::PropertyDeclaration::BackgroundColor(c) => { self.non_layout.background_color = *c; },
            css::PropertyDeclaration::BorderBottomColor(c) => { self.non_layout.border_color.bottom = *c; },
            css::PropertyDeclaration::BorderLeftColor(c) => { self.non_layout.border_color.left = *c; },
            css::PropertyDeclaration::BorderRightColor(c) => { self.non_layout.border_color.right = *c; },
            css::PropertyDeclaration::BorderTopColor(c) => { self.non_layout.border_color.top = *c; },
            css::PropertyDeclaration::BorderBottomWidth(w) => { self.non_layout.border_width.bottom = *w; },
            css::PropertyDeclaration::BorderLeftWidth(w) => { self.non_layout.border_width.left = *w; },
            css::PropertyDeclaration::BorderRightWidth(w) => { self.non_layout.border_width.right = *w; },
            css::PropertyDeclaration::BorderTopWidth(w) => { self.non_layout.border_width.top = *w; },
            css::PropertyDeclaration::BorderRadius(radius) => { self.non_layout.border_radius = *radius; },

            // Layout-altering styles
            css::PropertyDeclaration::AlignContent(v) => { self.layout.align_content = *v; },
            css::PropertyDeclaration::AlignItems(v) => { self.layout.align_items = *v; },
            css::PropertyDeclaration::AlignSelf(v) => { self.layout.align_self = *v; },
            css::PropertyDeclaration::Display(v) => { self.layout.display = *v; },
            css::PropertyDeclaration::FlexBasis(v) => { self.layout.flex_basis = *v; },
            css::PropertyDeclaration::FlexDirection(v) => { self.layout.flex_direction = *v; },
            css::PropertyDeclaration::FlexGrow(v) => { self.layout.flex_grow = *v; },
            css::PropertyDeclaration::FlexShrink(v) => {  self.layout.flex_shrink = *v; },
            css::PropertyDeclaration::FlexWrap(v) => {  self.layout.flex_wrap = *v; },
            css::PropertyDeclaration::JustifyContent(v) => {  self.layout.justify_content = *v; },
            css::PropertyDeclaration::MarginLeft(v) => {  self.layout.margin.left = *v; },
            css::PropertyDeclaration::MarginRight(v) => {  self.layout.margin.right = *v; },
            css::PropertyDeclaration::MarginTop(v) => {  self.layout.margin.top = *v; },
            css::PropertyDeclaration::MarginBottom(v) => {  self.layout.margin.bottom = *v; },
            css::PropertyDeclaration::MaxHeight(v) => {  self.layout.max_height = *v; },
            css::PropertyDeclaration::MaxWidth(v) => {  self.layout.max_width = *v; },
            css::PropertyDeclaration::MinHeight(v) => {  self.layout.min_height = *v; },
            css::PropertyDeclaration::MinWidth(v) => {  self.layout.min_width = *v; },
            css::PropertyDeclaration::Overflow(v) => {  self.layout.overflow = *v; },
            css::PropertyDeclaration::PaddingLeft(v) => {  self.layout.padding.left = *v; },
            css::PropertyDeclaration::PaddingRight(v) => {  self.layout.padding.right = *v; },
            css::PropertyDeclaration::PaddingTop(v) => {  self.layout.padding.top = *v; },
            css::PropertyDeclaration::PaddingBottom(v) => {  self.layout.padding.bottom = *v; },
            css::PropertyDeclaration::Position(v) => {  self.layout.position = *v; },

            // Dynamic layout
            css::PropertyDeclaration::Bottom(v) => { self.dyn_layout.bottom = *v; },
            css::PropertyDeclaration::Left(v) => {  self.dyn_layout.left = *v; },
            css::PropertyDeclaration::Right(v) => {  self.dyn_layout.right = *v; },
            css::PropertyDeclaration::Top(v) => {  self.dyn_layout.top = *v; },
            css::PropertyDeclaration::Width(v) => {  self.dyn_layout.width = *v; },
            css::PropertyDeclaration::Height(v) => {  self.dyn_layout.height = *v; },

            // Other
            _ => { unimplemented!() }
        }
    }


}

/// Calculated style.
/// Some components of style may be shared between items to reduce memory usage.
#[derive(Clone,Debug)]
pub struct CachedStyle
{
    pub non_layout: Rc<NonLayoutStyles>,
    pub non_layout_dirty: bool,
    pub layout: Rc<LayoutStyles>,
    /// Whether the layout has changed since last time.
    pub layout_dirty: bool,
    /// Not Rc since they vary often among elements.
    pub dyn_layout: DynamicLayoutStyles
}

impl CachedStyle
{
    /// Update from a calculated style.
    pub fn update(&mut self, computed: &ComputedStyle2) {

        if &*self.non_layout != &computed.non_layout {
            // TODO fetch style block from a cache
            *Rc::make_mut(&mut self.non_layout) = computed.non_layout.clone();
            self.non_layout_dirty = true;
        }
        if &*self.layout != &computed.layout {
            *Rc::make_mut(&mut self.layout) = computed.layout.clone();
            self.layout_dirty = true;
        }
        // update dyn layout unconditionally
        self.dyn_layout = computed.dyn_layout.clone();
    }
}

/*
macro_rules! impl_property_setter {
    ($block:ident,$block_dirty:ident,$prop:ident,$new:expr) => {
        if self.$block.$prop != $new {
            *Rc::make_mut(self.$block).$prop = $new.clone();
            self.$block_dirty = true;
        }
    };
}

macro_rules! impl_property_setter_2 {
    ($block:ident,$block_dirty:ident,$prop:ident . $prop2:ident,$new:expr) => {
        if self.$block.$prop.$prop2 != $new {
            *Rc::make_mut(self.$block).$prop = $new.clone();
            self.$block_dirty = true;
        }
    };
}*/

/*impl ComputedStyle2
{


    //
}*/

/// Border style
/*pub enum BorderStyle
{
    Default
}*/

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
/*#[derive(Clone, Debug)]
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
}*/

/*macro_rules! inherit_props {
    ($left:expr, $parent:expr, $($prop:ident),*) => {
        Style {
            $($prop: $left.$prop.clone().or($parent.$prop.clone()),)*
            .. $left.clone()
        }
    };
}*/


/*impl Style {
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
}*/

#[derive(Clone,Debug, PartialEq)]
pub struct BoxProperty<T: Clone+PartialEq>
{
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T
}

impl<T: Clone+PartialEq> BoxProperty<T>
{
    pub fn all(val: T) -> BoxProperty<T> {
        BoxProperty {
            top: val.clone(),
            right: val.clone(),
            bottom: val.clone(),
            left: val
        }
    }
}

/// Visual style
#[derive(Clone, Debug)]
pub struct ComputedStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_color: Color,
    pub background_color: Color,
    pub border_color: BoxProperty<Color>,
    pub border_width: BoxProperty<f32>,
    pub border_radius: f32,
    pub background: Option<Background>,

    pub display: yoga::Display,
    pub align_content: yoga::Align,
    pub align_items: yoga::Align,
    pub align_self: yoga::Align,
    pub aspect_ratio: f32,
    pub border_end: f32,
    pub position: yoga::PositionType,
    pub left: yoga::StyleUnit,
    pub right: yoga::StyleUnit,
    pub top: yoga::StyleUnit,
    pub bottom: yoga::StyleUnit,
    pub width: yoga::StyleUnit,
    pub height: yoga::StyleUnit,
    pub start: yoga::StyleUnit,
    pub end: yoga::StyleUnit,
    pub flex_basis: yoga::StyleUnit,
    pub flex_direction: yoga::FlexDirection,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_wrap: yoga::Wrap,
    pub justify_content: yoga::Justify,
    pub margin: BoxProperty<yoga::StyleUnit>,
    pub margin_end: yoga::StyleUnit,
    //pub margin_horizontal: yoga::StyleUnit,
    pub margin_start: yoga::StyleUnit,
    //pub margin_vertical: yoga::StyleUnit,
    pub max_height: yoga::StyleUnit,
    pub max_width: yoga::StyleUnit,
    pub min_height: yoga::StyleUnit,
    pub min_width: yoga::StyleUnit,
    pub overflow: yoga::Overflow,
    pub padding: BoxProperty<yoga::StyleUnit>,
    pub padding_end: yoga::StyleUnit,
    //pub padding_horizontal: yoga::StyleUnit,
    pub padding_start: yoga::StyleUnit,
    //pub padding_vertical: yoga::StyleUnit,
}

impl Default for ComputedStyle
{
    fn default() -> Self {
        ComputedStyle {
            font_family: "monospace".to_owned(),
            font_size: 12.0,
            font_color: (0.0,0.0,0.0,0.0),
            background_color: (0.0,0.0,0.0,0.0),
            border_color: BoxProperty::all((0.0,0.0,0.0,0.0)),
            border_width: BoxProperty::all(0.0),
            border_radius: 0.0,
            margin: BoxProperty::all(yoga::StyleUnit::UndefinedValue),
            margin_end: yoga::StyleUnit::UndefinedValue,
            //margin_horizontal: yoga::StyleUnit::UndefinedValue,
            margin_start: yoga::StyleUnit::UndefinedValue,
            //margin_vertical: yoga::StyleUnit::UndefinedValue,
            max_height: yoga::StyleUnit::UndefinedValue,
            max_width: yoga::StyleUnit::UndefinedValue,
            min_height: yoga::StyleUnit::UndefinedValue,
            min_width: yoga::StyleUnit::UndefinedValue,
            overflow: yoga::Overflow::Visible,
            padding: BoxProperty::all(yoga::StyleUnit::UndefinedValue),
            padding_end: yoga::StyleUnit::UndefinedValue,
            //padding_horizontal: yoga::StyleUnit::UndefinedValue,
            padding_start: yoga::StyleUnit::UndefinedValue,
            //padding_vertical: yoga::StyleUnit::UndefinedValue,
            position: yoga::PositionType::Relative,
            right: yoga::StyleUnit::UndefinedValue,
            start: yoga::StyleUnit::UndefinedValue,
            top: yoga::StyleUnit::UndefinedValue,
            background: None,
            align_content: yoga::Align::Auto,
            align_items: yoga::Align::Auto,
            align_self: yoga::Align::Auto,
            aspect_ratio: yoga::Undefined,  // == Auto
            border_end: 0.0,
            bottom: yoga::StyleUnit::UndefinedValue,
            left: yoga::StyleUnit::UndefinedValue,
            end: yoga::StyleUnit::UndefinedValue,
            display: yoga::Display::Flex,
            flex_basis: yoga::StyleUnit::UndefinedValue,
            flex_direction: yoga::FlexDirection::Column,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_wrap: yoga::Wrap::NoWrap,
            height: yoga::StyleUnit::UndefinedValue,
            justify_content: yoga::Justify::FlexStart,
            width: yoga::StyleUnit::UndefinedValue,
        }
    }
}

macro_rules! inherit_props_2 {
    ($to:expr, $from:expr, $($prop:ident),*) => {
        $($to.$prop = $from.$prop.clone();)*
    };
}

impl ComputedStyle
{
    pub fn apply(&mut self, properties: &[css::PropertyDeclaration], should_relayout: &mut bool) {
        for prop in properties.iter() {
            match prop {
                css::PropertyDeclaration::Color(c) => { unimplemented!() },
                css::PropertyDeclaration::BackgroundColor(c) => { self.background_color = *c; },
                css::PropertyDeclaration::BorderBottomColor(c) => { self.border_color.bottom = *c; },
                css::PropertyDeclaration::BorderLeftColor(c) => { self.border_color.left = *c; },
                css::PropertyDeclaration::BorderRightColor(c) => { self.border_color.right = *c; },
                css::PropertyDeclaration::BorderTopColor(c) => { self.border_color.top = *c; },
                css::PropertyDeclaration::BorderBottomWidth(w) => { self.border_width.bottom = *w; },
                css::PropertyDeclaration::BorderLeftWidth(w) => { self.border_width.left = *w; },
                css::PropertyDeclaration::BorderRightWidth(w) => { self.border_width.right = *w; },
                css::PropertyDeclaration::BorderTopWidth(w) => { self.border_width.top = *w; },
                css::PropertyDeclaration::BorderRadius(radius) => { self.border_radius = *radius; },
                /// Flex styles.
                //css::PropertyDeclaration::AspectRatio(),
                //css::PropertyDeclaration::BorderEnd(),
                css::PropertyDeclaration::AlignContent(v) => { *should_relayout = true; self.align_content = *v; },
                css::PropertyDeclaration::AlignItems(v) => { *should_relayout = true; self.align_items = *v; },
                css::PropertyDeclaration::AlignSelf(v) => { *should_relayout = true; self.align_self = *v; },
                css::PropertyDeclaration::Bottom(v) => { *should_relayout = true; self.bottom = *v; },
                css::PropertyDeclaration::Display(v) => { *should_relayout = true; self.display = *v; },
                css::PropertyDeclaration::End(v) => { *should_relayout = true; self.end = *v; },
                css::PropertyDeclaration::FlexBasis(v) => { *should_relayout = true; self.flex_basis = *v; },
                css::PropertyDeclaration::FlexDirection(v) => { *should_relayout = true; self.flex_direction = *v; },
                css::PropertyDeclaration::FlexGrow(v) => { *should_relayout = true; self.flex_grow = *v; },
                css::PropertyDeclaration::FlexShrink(v) => { *should_relayout = true; self.flex_shrink = *v; },
                css::PropertyDeclaration::FlexWrap(v) => { *should_relayout = true; self.flex_wrap = *v; },
                css::PropertyDeclaration::JustifyContent(v) => { *should_relayout = true; self.justify_content = *v; },
                //css::PropertyDeclaration::MarginHorizontal(v) => { *should_relayout = true; self.margin_horizontal = *v; },
                css::PropertyDeclaration::MarginLeft(v) => { *should_relayout = true; self.margin.left = *v; },
                css::PropertyDeclaration::MarginRight(v) => { *should_relayout = true; self.margin.right = *v; },
                css::PropertyDeclaration::MarginTop(v) => { *should_relayout = true; self.margin.top = *v; },
                css::PropertyDeclaration::MarginBottom(v) => { *should_relayout = true; self.margin.bottom = *v; },
                //css::PropertyDeclaration::MarginStart(v) => { *should_relayout = true; self.margin_start = *v; },
                //css::PropertyDeclaration::MarginVertical(v) => { *should_relayout = true; self.margin_vertical = *v; },
                //css::PropertyDeclaration::MarginEnd(v) => { *should_relayout = true; self.margin_end = *v; },
                css::PropertyDeclaration::MaxHeight(v) => { *should_relayout = true; self.max_height = *v; },
                css::PropertyDeclaration::MaxWidth(v) => { *should_relayout = true; self.max_width = *v; },
                css::PropertyDeclaration::MinHeight(v) => { *should_relayout = true; self.min_height = *v; },
                css::PropertyDeclaration::MinWidth(v) => { *should_relayout = true; self.min_width = *v; },
                css::PropertyDeclaration::Overflow(v) => { *should_relayout = true; self.overflow = *v; },
                //css::PropertyDeclaration::PaddingEnd(v) => { *should_relayout = true; self.padding_end = *v; },
                //css::PropertyDeclaration::PaddingHorizontal(v) => { *should_relayout = true; self.padding_horizontal = *v; },
                css::PropertyDeclaration::PaddingLeft(v) => { *should_relayout = true; self.padding.left = *v; },
                css::PropertyDeclaration::PaddingRight(v) => { *should_relayout = true; self.padding.right = *v; },
                css::PropertyDeclaration::PaddingTop(v) => { *should_relayout = true; self.padding.top = *v; },
                css::PropertyDeclaration::PaddingBottom(v) => { *should_relayout = true; self.padding.bottom = *v; },
                //css::PropertyDeclaration::PaddingStart(v) => { *should_relayout = true; self.padding_start = *v; },
                //css::PropertyDeclaration::PaddingVertical(v) => { *should_relayout = true; self.padding_vertical = *v; },
                css::PropertyDeclaration::Position(v) => { *should_relayout = true; self.position = *v; },
                css::PropertyDeclaration::Left(v) => { *should_relayout = true; self.left = *v; },
                css::PropertyDeclaration::Right(v) => { *should_relayout = true; self.right = *v; },
                css::PropertyDeclaration::Start(v) => { *should_relayout = true; self.start = *v; },
                css::PropertyDeclaration::Top(v) => { *should_relayout = true; self.top = *v; },
                css::PropertyDeclaration::Width(v) => { *should_relayout = true; self.width = *v; },
                css::PropertyDeclaration::Height(v) => { *should_relayout = true; self.height = *v; },
                _ => { unimplemented!() }
            }
        }
    }

    pub fn inherit(&mut self, from: &ComputedStyle) -> &mut Self {
        inherit_props_2!(self, from, font_family, font_size, font_color);
        self
    }
}

pub(super) fn apply_to_flex_node(node: &mut yoga::Node, style: &ComputedStyle)
{
    // TODO rewrite this with direct calls to methods of Node
    let styles = &[
        yoga::FlexStyle::AlignContent(style.align_content),
        yoga::FlexStyle::AlignItems(style.align_items),
        yoga::FlexStyle::AlignSelf(style.align_self),
        yoga::FlexStyle::AspectRatio(style.aspect_ratio.into()),
        yoga::FlexStyle::BorderEnd(style.border_end.into()),
        yoga::FlexStyle::Left(style.left),
        yoga::FlexStyle::Right(style.right),
        yoga::FlexStyle::Top(style.top),
        yoga::FlexStyle::Bottom(style.bottom),
        yoga::FlexStyle::Width(style.width),  // set by measure
        yoga::FlexStyle::Height(style.height),    // set by measure
        yoga::FlexStyle::Start(style.start),
        yoga::FlexStyle::End(style.end),
        yoga::FlexStyle::Display(style.display),
        yoga::FlexStyle::FlexBasis(style.flex_basis),
        yoga::FlexStyle::FlexDirection(style.flex_direction),
        yoga::FlexStyle::FlexGrow(style.flex_grow.into()),
        yoga::FlexStyle::FlexShrink(style.flex_shrink.into()),
        yoga::FlexStyle::FlexWrap(style.flex_wrap),
        yoga::FlexStyle::JustifyContent(style.justify_content),

        yoga::FlexStyle::MarginTop(style.margin.top),
        yoga::FlexStyle::MarginBottom(style.margin.bottom),
        yoga::FlexStyle::MarginLeft(style.margin.left),
        yoga::FlexStyle::MarginRight(style.margin.right),

        yoga::FlexStyle::PaddingTop(style.padding.top),
        yoga::FlexStyle::PaddingBottom(style.padding.bottom),
        yoga::FlexStyle::PaddingLeft(style.padding.left),
        yoga::FlexStyle::PaddingRight(style.padding.right),

        yoga::FlexStyle::MarginEnd(style.margin_end),
        yoga::FlexStyle::MarginStart(style.margin_start),
        yoga::FlexStyle::MaxHeight(style.max_height),
        yoga::FlexStyle::MaxWidth(style.max_width),
        yoga::FlexStyle::MinHeight(style.min_height),
        yoga::FlexStyle::MinWidth(style.min_width),
        yoga::FlexStyle::Overflow(style.overflow),
        yoga::FlexStyle::PaddingEnd(style.padding_end),
        yoga::FlexStyle::PaddingStart(style.padding_start),
        yoga::FlexStyle::Position(style.position),
    ];
    node.apply_styles(&styles[..]);
}
