use super::css;
use yoga;
use warmy::{FSKey, Res, Store, StoreOpt};

use std::rc::Rc;
use std::collections::{HashMap, hash_map::{Entry, OccupiedEntry, VacantEntry}};

/// Font description
#[derive(Clone, Debug)]
pub struct FontDesc(String);

pub type Color = (f32, f32, f32, f32);

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

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum BoxShadowClipMode
{
    Inset,
    Outset,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadow
{
    pub color: Color,
    pub horizontal_offset: f32,
    pub vertical_offset: f32,
    pub blur_radius: f32,
    pub spread: f32,
    pub clip_mode: BoxShadowClipMode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoxProperty<T: Clone + PartialEq> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Clone + PartialEq> BoxProperty<T> {
    pub fn all(val: T) -> BoxProperty<T> {
        BoxProperty {
            top: val.clone(),
            right: val.clone(),
            bottom: val.clone(),
            left: val,
        }
    }
}

/// Calculated border and background style.
/// Changing these should not trigger a full relayout.
#[derive(Clone, Debug, PartialEq)]
pub struct NonLayoutStyles {
    pub background_color: Color,
    pub border_color: BoxProperty<Color>,
    pub border_width: BoxProperty<f32>,
    pub border_radius: f32,
    pub box_shadow: Option<BoxShadow>
}

impl Default for NonLayoutStyles {
    fn default() -> NonLayoutStyles {
        NonLayoutStyles {
            background_color: (0.0, 0.0, 0.0, 0.0),
            border_color: BoxProperty::all((0.0, 0.0, 0.0, 0.0)),
            border_width: BoxProperty::all(0.0),
            border_radius: 0.0,
            box_shadow: None
        }
    }
}

/// Calculated layout properties.
/// These properties can trigger a relayout when changed and are not meant to change frequently.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutStyles {
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

impl Default for LayoutStyles {
    fn default() -> LayoutStyles {
        LayoutStyles {
            display: yoga::Display::Flex,
            align_content: yoga::Align::FlexStart,
            align_items: yoga::Align::Stretch,
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
            overflow: yoga::Overflow::Hidden,
            padding: BoxProperty::all(yoga::StyleUnit::UndefinedValue),
        }
    }
}

/// Dynamic layout properties.
/// These are expected to change more frequently than the other layout properties,
/// but they do not trigger a full relayout.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DynamicLayoutStyles {
    pub left: yoga::StyleUnit,
    pub right: yoga::StyleUnit,
    pub top: yoga::StyleUnit,
    pub bottom: yoga::StyleUnit,
    pub width: yoga::StyleUnit,
    pub height: yoga::StyleUnit,
}

impl Default for DynamicLayoutStyles {
    fn default() -> DynamicLayoutStyles {
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

/// Inherited font styles.
#[derive(Clone, Debug, PartialEq)]
pub struct FontStyles {
    pub font_family: String,
    pub font_size: f32,
    pub font_color: Color,
}

impl Default for FontStyles {
    fn default() -> FontStyles {
        FontStyles {
            font_family: "monospace".to_owned(),
            font_size: 12.0,
            font_color: (0.0, 0.0, 0.0, 0.0),
        }
    }
}

/// Struct that contains all the computed style properties of a node.
/// Note that computed properties are shared between nodes that have the same style selector.
#[derive(Debug)]
pub struct Styles {
    pub font: FontStyles,
    pub non_layout: NonLayoutStyles,
    pub layout: LayoutStyles,
    pub dyn_layout: DynamicLayoutStyles,
}

impl Default for Styles {
    fn default() -> Styles {
        Styles {
            font: Default::default(),
            non_layout: Default::default(),
            layout: Default::default(),
            dyn_layout: Default::default(),
        }
    }
}

impl Styles {
    /// Apply CSS property over these styles.
    pub(super) fn apply_property(&mut self, prop: &css::PropertyDeclaration) {
        match prop {
            // Non-layout styles
            css::PropertyDeclaration::Color(c) => unimplemented!(),
            css::PropertyDeclaration::BackgroundColor(c) => {
                self.non_layout.background_color = *c;
            }
            css::PropertyDeclaration::BorderBottomColor(c) => {
                self.non_layout.border_color.bottom = *c;
            }
            css::PropertyDeclaration::BorderLeftColor(c) => {
                self.non_layout.border_color.left = *c;
            }
            css::PropertyDeclaration::BorderRightColor(c) => {
                self.non_layout.border_color.right = *c;
            }
            css::PropertyDeclaration::BorderTopColor(c) => {
                self.non_layout.border_color.top = *c;
            }
            css::PropertyDeclaration::BorderBottomWidth(w) => {
                self.non_layout.border_width.bottom = *w;
            }
            css::PropertyDeclaration::BorderLeftWidth(w) => {
                self.non_layout.border_width.left = *w;
            }
            css::PropertyDeclaration::BorderRightWidth(w) => {
                self.non_layout.border_width.right = *w;
            }
            css::PropertyDeclaration::BorderTopWidth(w) => {
                self.non_layout.border_width.top = *w;
            }
            css::PropertyDeclaration::BorderRadius(radius) => {
                self.non_layout.border_radius = *radius;
            }
            css::PropertyDeclaration::BoxShadow(box_shadow) => {
                self.non_layout.box_shadow = box_shadow.clone();
            }

            // Layout-altering styles
            css::PropertyDeclaration::AlignContent(v) => {
                self.layout.align_content = *v;
            }
            css::PropertyDeclaration::AlignItems(v) => {
                self.layout.align_items = *v;
            }
            css::PropertyDeclaration::AlignSelf(v) => {
                self.layout.align_self = *v;
            }
            /*css::PropertyDeclaration::Display(v) => {
                self.layout.display = *v;
            }*/
            /*css::PropertyDeclaration::FlexBasis(v) => {
                self.layout.flex_basis = *v;
            }*/
            css::PropertyDeclaration::FlexDirection(v) => {
                self.layout.flex_direction = *v;
            }
            css::PropertyDeclaration::FlexGrow(v) => {
                self.layout.flex_grow = *v;
            }
            css::PropertyDeclaration::FlexShrink(v) => {
                self.layout.flex_shrink = *v;
            }
            /*css::PropertyDeclaration::FlexWrap(v) => {
                self.layout.flex_wrap = *v;
            }*/
            css::PropertyDeclaration::JustifyContent(v) => {
                self.layout.justify_content = *v;
            }
            css::PropertyDeclaration::MarginLeft(v) => {
                self.layout.margin.left = *v;
            }
            css::PropertyDeclaration::MarginRight(v) => {
                self.layout.margin.right = *v;
            }
            css::PropertyDeclaration::MarginTop(v) => {
                self.layout.margin.top = *v;
            }
            css::PropertyDeclaration::MarginBottom(v) => {
                self.layout.margin.bottom = *v;
            }
            /*css::PropertyDeclaration::MaxHeight(v) => {
                self.layout.max_height = *v;
            }
            css::PropertyDeclaration::MaxWidth(v) => {
                self.layout.max_width = *v;
            }
            css::PropertyDeclaration::MinHeight(v) => {
                self.layout.min_height = *v;
            }
            css::PropertyDeclaration::MinWidth(v) => {
                self.layout.min_width = *v;
            }*/
            /*css::PropertyDeclaration::Overflow(v) => {
                self.layout.overflow = *v;
            }*/
            css::PropertyDeclaration::PaddingLeft(v) => {
                self.layout.padding.left = *v;
            }
            css::PropertyDeclaration::PaddingRight(v) => {
                self.layout.padding.right = *v;
            }
            css::PropertyDeclaration::PaddingTop(v) => {
                self.layout.padding.top = *v;
            }
            css::PropertyDeclaration::PaddingBottom(v) => {
                self.layout.padding.bottom = *v;
            }
            css::PropertyDeclaration::Position(v) => {
                self.layout.position = *v;
            }

            // Dynamic layout
            css::PropertyDeclaration::Bottom(v) => {
                self.dyn_layout.bottom = *v;
            }
            css::PropertyDeclaration::Left(v) => {
                self.dyn_layout.left = *v;
            }
            css::PropertyDeclaration::Right(v) => {
                self.dyn_layout.right = *v;
            }
            css::PropertyDeclaration::Top(v) => {
                self.dyn_layout.top = *v;
            }
            css::PropertyDeclaration::Width(v) => {
                self.dyn_layout.width = *v;
            }
            css::PropertyDeclaration::Height(v) => {
                self.dyn_layout.height = *v;
            }

            // Other
            _ => unimplemented!(),
        }
    }
}

/*/// Calculated style.
/// Some components of style may be shared between items to reduce memory usage.
#[derive(Clone, Debug)]
pub struct CachedStyle {
    pub font: Rc<FontStyles>,
    pub non_layout: Rc<NonLayoutStyles>,
    pub layout: Rc<LayoutStyles>,
    /// Not Rc since they vary often among elements.
    pub dyn_layout: DynamicLayoutStyles,
}

impl Default for CachedStyle {
    fn default() -> CachedStyle {
        CachedStyle {
            font: Rc::new(Default::default()),
            non_layout: Rc::new(Default::default()),
            layout: Rc::new(Default::default()),
            dyn_layout: Default::default(),
        }
    }
}

impl CachedStyle {
    /// Updates from a calculated style.
    /// Returns true if the flexbox layout was damaged.
    pub fn update(&mut self, computed: &ComputedStyle) -> bool {
        let mut layout_damaged = false;
        if &*self.non_layout != &computed.non_layout {
            // TODO fetch style block from a cache
            *Rc::make_mut(&mut self.non_layout) = computed.non_layout.clone();
        }
        if &*self.layout != &computed.layout {
            *Rc::make_mut(&mut self.layout) = computed.layout.clone();
            layout_damaged = true;
        }
        // update dyn layout unconditionally
        self.dyn_layout = computed.dyn_layout.clone();
        layout_damaged
    }
}
*/

/// Applies the styles relevant to flexbox layout to the specified yoga node.
pub(super) fn apply_to_flex_node(node: &mut yoga::Node, style: &Styles) {
    // TODO rewrite this with direct calls to methods of Node
    let styles = &[
        yoga::FlexStyle::AlignContent(style.layout.align_content),
        yoga::FlexStyle::AlignItems(style.layout.align_items),
        yoga::FlexStyle::AlignSelf(style.layout.align_self),
        yoga::FlexStyle::AspectRatio(style.layout.aspect_ratio.into()),
        //yoga::FlexStyle::BorderEnd(style.layout.border_end.into()),
        yoga::FlexStyle::Left(style.dyn_layout.left),
        yoga::FlexStyle::Right(style.dyn_layout.right),
        yoga::FlexStyle::Top(style.dyn_layout.top),
        yoga::FlexStyle::Bottom(style.dyn_layout.bottom),
        yoga::FlexStyle::Width(style.dyn_layout.width), // set by measure
        yoga::FlexStyle::Height(style.dyn_layout.height), // set by measure
        //yoga::FlexStyle::Start(style.layout.start),
        //yoga::FlexStyle::End(style.layout.end),
        yoga::FlexStyle::Display(style.layout.display),
        yoga::FlexStyle::FlexBasis(style.layout.flex_basis),
        yoga::FlexStyle::FlexDirection(style.layout.flex_direction),
        yoga::FlexStyle::FlexGrow(style.layout.flex_grow.into()),
        yoga::FlexStyle::FlexShrink(style.layout.flex_shrink.into()),
        yoga::FlexStyle::FlexWrap(style.layout.flex_wrap),
        yoga::FlexStyle::JustifyContent(style.layout.justify_content),
        yoga::FlexStyle::MarginTop(style.layout.margin.top),
        yoga::FlexStyle::MarginBottom(style.layout.margin.bottom),
        yoga::FlexStyle::MarginLeft(style.layout.margin.left),
        yoga::FlexStyle::MarginRight(style.layout.margin.right),
        yoga::FlexStyle::PaddingTop(style.layout.padding.top),
        yoga::FlexStyle::PaddingBottom(style.layout.padding.bottom),
        yoga::FlexStyle::PaddingLeft(style.layout.padding.left),
        yoga::FlexStyle::PaddingRight(style.layout.padding.right),
        // yoga::FlexStyle::MarginEnd(style.layout.margin_end),
        // yoga::FlexStyle::MarginStart(style.layout.margin_start),
        yoga::FlexStyle::MaxHeight(style.layout.max_height),
        yoga::FlexStyle::MaxWidth(style.layout.max_width),
        yoga::FlexStyle::MinHeight(style.layout.min_height),
        yoga::FlexStyle::MinWidth(style.layout.min_width),
        yoga::FlexStyle::Overflow(style.layout.overflow),
        // yoga::FlexStyle::PaddingEnd(style.layout.padding_end),
        // yoga::FlexStyle::PaddingStart(style.layout.padding_start),
        yoga::FlexStyle::Position(style.layout.position),
    ];
    node.apply_styles(&styles[..]);
}

/// Holds styles that are loaded from a CSS stylesheet.
pub struct StyleCache
{
    cache: HashMap<css::Selector, Rc<Styles>>
}

impl StyleCache
{
    /// Creates a new style cache.
    pub fn new() -> StyleCache
    {
        StyleCache {
            cache: HashMap::new()
        }
    }

    /// Invalidates all styles in the cache (clears the cache).
    pub fn invalidate(&mut self)
    {
        self.cache.clear();
    }

    /// Computes the style from the given selector. If the style is not already present
    /// in the cache, compute it from the provided stylesheets.
    pub fn get_styles(&mut self, stylesheets: &[Res<css::Stylesheet>], selector: css::Selector) -> Rc<Styles>
    {
        self.cache.entry(selector.clone()).or_insert_with(|| {
            // style not found in cache: compute it
            let mut styles = Styles::default();
            // look into all stylesheets, in the provided order ...
            for stylesheet in stylesheets.iter() {
                let stylesheet = stylesheet.borrow();
                // ... for rules that match the provided selector.
                // TODO actually fetch all rules?
                let rules = stylesheet.match_rules(&selector);
                if rules.is_empty() {
                    debug!("no rules for selector {:?}", selector);
                }
                // Now, apply all matching rules incrementally over the computed style
                for rule in rules {
                    debug!("(selector {:?}) rule {:?}", selector, rule);
                    // apply all declarations in the rule
                    for d in rule.declarations.iter() {
                        styles.apply_property(d);
                    }
                }
            }
            // wrap in Rc and add to cache
            Rc::new(styles)
        }).clone()
    }
}