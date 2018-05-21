use cssparser::{Parser,ParserInput,DeclarationListParser,DeclarationParser,RuleListParser,QualifiedRuleParser,CowRcStr,ParseError,AtRuleParser};
use cssparser::Color as CssColor;
use cssparser::{Token,RGBA};
use failure::{Fail,Error,Compat};
use yoga;
use yoga::prelude::*;
use warmy::{Load, FSKey, Storage, Loaded};
use std::io;

use super::style::*;

/// A CSS declaration
#[derive(Clone,Debug)]
pub enum PropertyDeclaration
{
    /// Color.
    Color(Color),
    /// Background color.
    BackgroundColor(Color),
    /// Borders.
    BorderBottomColor(Color),
    /// Borders.
    BorderLeftColor(Color),
    /// Borders.
    BorderRightColor(Color),
    /// Borders.
    BorderTopColor(Color),
    /// Borders.
    BorderBottomWidth(f32),
    /// Borders.
    BorderLeftWidth(f32),
    /// Borders.
    BorderRightWidth(f32),
    /// Borders.
    BorderTopWidth(f32),
    /// Borders.
    BorderRadius(f32),
    /// Flex styles.
    AlignContent(yoga::Align),
    AlignItems(yoga::Align),
    AlignSelf(yoga::Align),
    AspectRatio(f32),
    BorderEnd(f32),
    Bottom(yoga::StyleUnit),
    Display(yoga::Display),
    End(yoga::StyleUnit),
    Flex(f32),
    FlexBasis(yoga::StyleUnit),
    FlexDirection(yoga::FlexDirection),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexWrap(yoga::Wrap),
    Height(yoga::StyleUnit),
    JustifyContent(yoga::Justify),
    Left(yoga::StyleUnit),
    MarginBottom(yoga::StyleUnit),
    MarginEnd(yoga::StyleUnit),
    MarginHorizontal(yoga::StyleUnit),
    MarginLeft(yoga::StyleUnit),
    MarginRight(yoga::StyleUnit),
    MarginStart(yoga::StyleUnit),
    MarginTop(yoga::StyleUnit),
    MarginVertical(yoga::StyleUnit),
    MaxHeight(yoga::StyleUnit),
    MaxWidth(yoga::StyleUnit),
    MinHeight(yoga::StyleUnit),
    MinWidth(yoga::StyleUnit),
    Overflow(yoga::Overflow),
    PaddingBottom(yoga::StyleUnit),
    PaddingEnd(yoga::StyleUnit),
    PaddingHorizontal(yoga::StyleUnit),
    PaddingLeft(yoga::StyleUnit),
    PaddingRight(yoga::StyleUnit),
    PaddingStart(yoga::StyleUnit),
    PaddingTop(yoga::StyleUnit),
    PaddingVertical(yoga::StyleUnit),
    Position(yoga::PositionType),
    Right(yoga::StyleUnit),
    Start(yoga::StyleUnit),
    Top(yoga::StyleUnit),
    Width(yoga::StyleUnit),

}


/// A CSS selector.
#[derive(Clone,Debug)]
pub struct Selector
{
    /// TODO
    class: String
}

/// A CSS rule-set.
#[derive(Clone,Debug)]
pub struct Rule
{
    /// Selector.
    pub(super) selector: Selector,
    /// List of CSS declarations.
    pub(super) declarations: Vec<PropertyDeclaration>
}

/// A stylesheet.
#[derive(Clone,Debug)]
pub struct Stylesheet
{
    /// List of rule-sets.
    rules: Vec<Rule>
}

impl Stylesheet
{
    pub fn match_class(&self, class: &str) -> Option<&Rule> {
        // TODO
        self.rules.iter().filter(|rule| rule.selector.class == class).next()
    }
}

#[derive(Debug,Fail)]
pub enum StylesheetLoadError
{
    #[fail(display = "io error")]
    IoError(io::Error),
    #[fail(display = "parse error")]
    ParseError(Compat<Error>)
}

/// Hot-reloadable impl.
impl<C> Load<C> for Stylesheet {
    type Key = FSKey;
    type Error = Compat<Error>;

    fn load(key: Self::Key, storage: &mut Storage<C>, ctx: &mut C) -> Result<Loaded<Self>, Self::Error> {
        use std::fs;
        let src = fs::read_to_string(key.as_path()).map_err(|e| Error::from(e).compat())?;
        let stylesheet = parse_stylesheet(&src).map_err(|e| e.compat())?;
        debug!("(re)-loaded stylesheet `{}`", key.as_path().display());
        Ok(stylesheet.into())
    }
}

////////////////////////////////////////////////////////////////
// PARSER
////////////////////////////////////////////////////////////////

struct RulesParser;

#[derive(Copy,Clone,Debug)]
enum RuleParseErrorKind {
    Other
}

impl<'i> AtRuleParser<'i> for RulesParser {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Rule;
    type Error = RuleParseErrorKind;
}

impl<'i> QualifiedRuleParser<'i> for RulesParser {
    type Prelude = Selector;
    type QualifiedRule = Rule;
    type Error = RuleParseErrorKind;

    /// Parse the selector (only a single class identifier for now)
    fn parse_prelude<'t>(&mut self, parser: &mut Parser<'i, 't>)
                         -> Result<Self::Prelude, ParseError<'i, Self::Error>>
    {
        Ok(parser.expect_ident().map(|ident| Selector { class: ident.to_string() })?)
    }

    /// Parse the declaration block.
    fn parse_block<'t>(&mut self, prelude: Self::Prelude, parser: &mut Parser<'i, 't>)
                       -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>>
    {
        let mut decl_list_parser = DeclarationListParser::new(parser, PropertyDeclarationParser::new());

        while let Some(result) = decl_list_parser.next() {
            match result {
                Ok(()) => {
                    // Got a decl
                }
                Err(e) => {
                    // FIXME error reporting
                    warn!("Error parsing CSS declaration: {:?}", e);
                }
            }
        }

        let rule = Rule {
            declarations: decl_list_parser.parser.declarations,
            selector: prelude,
        };

        Ok(rule)
    }
}

/// Trait for converting a cssparser::Color to a Color (f32*4 tuple)
trait ToColor {
    fn to_color(&self) -> Color;
}

impl ToColor for CssColor
{
    fn to_color(&self) -> Color {
        match self {
            &CssColor::CurrentColor => { unimplemented!() },
            &CssColor::RGBA(RGBA { red, green, blue, alpha }) => { (red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0, alpha as f32 / 255.0) }
        }
    }
}

#[derive(Clone,Debug)]
enum PropertyParseErrorKind<'i> {
    UnknownProperty(CowRcStr<'i>),
    UnsupportedUnit,
    Other
}

/// Length values.
#[derive(Copy,Clone,Debug)]
enum Length {
    Px(f32)
}

trait ToPx
{
    fn to_px<'i>(&self) -> Option<f32>;
}

impl ToPx for yoga::StyleUnit
{

    fn to_px<'i>(&self) -> Option<f32> {
        match self {
            &yoga::StyleUnit::Point(px) => Some(px.into()),
            _ => { None }
        }
    }
}

impl Length
{
    fn parse<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Length, ParseError<'i, PropertyParseErrorKind<'i>>> {
        match *parser.next()? {
            Token::Dimension { value, ref unit, .. } => match unit.as_ref() {
                "px" => return Ok(Length::Px(value)),
                _ => {}
            },
            _ => {}
        }
        Err(parser.new_custom_error(PropertyParseErrorKind::Other))
    }

}

fn parse_style_unit<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<yoga::StyleUnit, ParseError<'i, PropertyParseErrorKind<'i>>> {
    match parser.next()?.clone() {
        Token::Dimension { value, ref unit, .. } => match unit.as_ref() {
            "px" => Ok(value.point()),
            "pt" => Ok(value.point()),
            _ => { Err(parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit)) }
        },
        Token::Percentage { unit_value, .. } => {
            Ok(unit_value.percent())
        },
        _ => { Err(parser.new_custom_error(PropertyParseErrorKind::Other)) }
    }
}

/// Box property parser (e.g. border-width: 5px 10px 5px).
/// Result is top, right, bottom, left
fn parse_box_width<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<(yoga::StyleUnit,yoga::StyleUnit,yoga::StyleUnit,yoga::StyleUnit), ParseError<'i, PropertyParseErrorKind<'i>>>
{
    let a0 = parse_style_unit(parser)?;
    // the others may fail
    let a1 = parse_style_unit(parser).ok();
    let a2 = parse_style_unit(parser).ok();
    let a3 = parse_style_unit(parser).ok();

    match (a1,a2,a3) {
        (None, None, None) => Ok((a0,a0,a0,a0)),
        (Some(b1), None, None) => Ok((a0, b1, a0, b1)),
        (Some(b1), Some(b2), None) => Ok((a0, b1, b2, b1)),
        (Some(b1), Some(b2), Some(b3)) => Ok((a0, b1, b2, b3)),
        _ => { unreachable!() }
    }
}

struct PropertyDeclarationParser {
    declarations: Vec<PropertyDeclaration>,
}

impl PropertyDeclarationParser {
    fn new() -> PropertyDeclarationParser {
        PropertyDeclarationParser {
            declarations: Vec::new()
        }
    }
}

impl<'i> AtRuleParser<'i> for PropertyDeclarationParser {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = ();
    type Error = PropertyParseErrorKind<'i>;
}


impl<'i> DeclarationParser<'i> for PropertyDeclarationParser {
    type Declaration = ();
    type Error = PropertyParseErrorKind<'i>;

    fn parse_value<'t>(&mut self, name: CowRcStr<'i>, parser: &mut Parser<'i, 't>)
                       -> Result<Self::Declaration, ParseError<'i, Self::Error>>
    {
        use cssparser::RGBA;

        match name.as_ref() {
            "color" => {
                self.declarations.push(PropertyDeclaration::Color(CssColor::parse(parser)?.to_color()));
                Ok(())
            },
            "border-color" => {
                let color = CssColor::parse(parser)?.to_color();
                self.declarations.push(PropertyDeclaration::BorderBottomColor(color));
                self.declarations.push(PropertyDeclaration::BorderTopColor(color));
                self.declarations.push(PropertyDeclaration::BorderRightColor(color));
                self.declarations.push(PropertyDeclaration::BorderLeftColor(color));
                Ok(())
            },
            "border-width" => {
                let (top, right, bottom, left) = parse_box_width(parser)?;
                //let width = Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?;
                self.declarations.push(PropertyDeclaration::BorderBottomWidth(bottom.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?));
                self.declarations.push(PropertyDeclaration::BorderTopWidth(top.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?));
                self.declarations.push(PropertyDeclaration::BorderRightWidth(right.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?));
                self.declarations.push(PropertyDeclaration::BorderLeftWidth(left.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?));
                Ok(())
            },
            "margin" => {
                let (top, right, bottom, left) = parse_box_width(parser)?;
                //let width = Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?;
                self.declarations.push(PropertyDeclaration::MarginBottom(bottom));
                self.declarations.push(PropertyDeclaration::MarginTop(top));
                self.declarations.push(PropertyDeclaration::MarginRight(right));
                self.declarations.push(PropertyDeclaration::MarginLeft(left));
                Ok(())
            },
            "padding" => {
                let (top, right, bottom, left) = parse_box_width(parser)?;
                //let width = Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?;
                self.declarations.push(PropertyDeclaration::PaddingBottom(bottom));
                self.declarations.push(PropertyDeclaration::PaddingTop(top));
                self.declarations.push(PropertyDeclaration::PaddingRight(right));
                self.declarations.push(PropertyDeclaration::PaddingLeft(left));
                Ok(())
            },
            "border-bottom-width" => { self.declarations.push(PropertyDeclaration::BorderBottomWidth(parse_style_unit(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-top-width" => { self.declarations.push(PropertyDeclaration::BorderTopWidth(parse_style_unit(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-left-width" => { self.declarations.push(PropertyDeclaration::BorderLeftWidth(parse_style_unit(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-right-width" => { self.declarations.push(PropertyDeclaration::BorderRightWidth(parse_style_unit(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-radius" => { self.declarations.push(PropertyDeclaration::BorderRadius(parse_style_unit(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "background-color" => {
                self.declarations.push(PropertyDeclaration::BackgroundColor(CssColor::parse(parser)?.to_color()));
                Ok(())
            },
            // flexbox properties
            "flex-direction" => {
                let ident = parser.expect_ident_cloned()?;
                let dir = match ident.as_ref() {
                    "row" => { yoga::FlexDirection::Row },
                    "row-reverse" => { yoga::FlexDirection::RowReverse },
                    "column" => { yoga::FlexDirection::Column },
                    "column-reverse" => { yoga::FlexDirection::ColumnReverse },
                    _ => return Err(parser.new_custom_error(PropertyParseErrorKind::Other))
                };
                self.declarations.push(PropertyDeclaration::FlexDirection(dir));
                Ok(())
            },
            "justify-content" => {
                let ident = parser.expect_ident_cloned()?;
                let justify = match ident.as_ref() {
                    "flex-start" => { yoga::Justify::FlexStart },
                    "flex-end" => { yoga::Justify::FlexEnd },
                    "center" => { yoga::Justify::Center },
                    "space-between" => { yoga::Justify::SpaceBetween },
                    "space-around" => { yoga::Justify::SpaceAround },
                    _ => return Err(parser.new_custom_error(PropertyParseErrorKind::Other))
                };
                self.declarations.push(PropertyDeclaration::JustifyContent(justify));
                Ok(())
            },
            "align-items" | "align-self" | "align-content" => {
                let ident = parser.expect_ident_cloned()?;
                let align = match ident.as_ref() {
                    "flex-start" => { yoga::Align::FlexStart },
                    "flex-end" => { yoga::Align::FlexEnd },
                    "center" => { yoga::Align::Center },
                    "baseline" => { yoga::Align::Baseline },
                    "stretch" => { yoga::Align::Stretch },
                    _ => return Err(parser.new_custom_error(PropertyParseErrorKind::Other))
                };
                match name.as_ref() {
                    "align-items" => {self.declarations.push(PropertyDeclaration::AlignItems(align));},
                    "align-self" => {self.declarations.push(PropertyDeclaration::AlignSelf(align));},
                    "align-content" => {self.declarations.push(PropertyDeclaration::AlignContent(align));},
                    _ => unreachable!()
                }
                Ok(())
            },
            "flex-grow" => {
                let grow = parser.expect_number()?;
                self.declarations.push(PropertyDeclaration::FlexGrow(grow.into()));
                debug!("flex-grow {}", grow);
                Ok(())
            },
            "flex-shrink" => {
                let shrink = parser.expect_number()?;
                self.declarations.push(PropertyDeclaration::FlexShrink(shrink.into()));
                Ok(())
            },
            "flex-basis" => {
                unimplemented!()
            },
            "position" => {
                let ident = parser.expect_ident_cloned()?;
                let pos = match ident.as_ref() {
                    "absolute" => { yoga::PositionType::Absolute },
                    "relative" => { yoga::PositionType::Relative },
                    _ => return Err(parser.new_custom_error(PropertyParseErrorKind::Other))
                };
                self.declarations.push(PropertyDeclaration::Position(pos));
                Ok(())
            },
            "width" => {
                let w = parse_style_unit(parser)?;
                self.declarations.push(PropertyDeclaration::Width(w));
                Ok(())
            },
            "height" => {
                let h = parse_style_unit(parser)?;
                self.declarations.push(PropertyDeclaration::Height(h));
                Ok(())
            },
            _ => Err(parser.new_custom_error(PropertyParseErrorKind::UnknownProperty(name)))
        }

    }
}

/// Parse a stylesheet.
pub(super) fn parse_stylesheet(text: &str) -> Result<Stylesheet, Error>
{
    // create the parser input
    let mut input = ParserInput::new(text);
    // create the parser
    let mut parser = Parser::new(&mut input);
    // list of errors
    //let mut errors = Vec::new();
    // stylesheet
    let mut stylesheet = Stylesheet {
        rules: Vec::new()
    };

    // parse a list of rules
    for result in RuleListParser::new_for_stylesheet(&mut parser, RulesParser) {
        match result {
            Ok(rule) => {
                stylesheet.rules.push(rule)
            }
            Err(e) => {
                warn!("Error parsing CSS rule")
                // TODO
            }
        }
    }

    Ok(stylesheet)
}
