use cssparser::{Parser,ParserInput,DeclarationListParser,DeclarationParser,RuleListParser,QualifiedRuleParser,CowRcStr,ParseError,AtRuleParser};
use cssparser::Color as CssColor;
use cssparser::{Token,RGBA};
use failure::Error;
use yoga::FlexStyle;

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
    /// Flexbox styles.
    Flexbox(FlexStyle)
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
    selector: Selector,
    /// List of CSS declarations.
    declarations: Vec<PropertyDeclaration>
}

/// A stylesheet.
#[derive(Clone,Debug)]
pub struct Stylesheet
{
    /// List of rule-sets.
    rules: Vec<Rule>
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
enum Length {
    Px(f32)
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

    fn to_px<'i>(&self) -> Option<f32> {
        match self {
            &Length::Px(px) => Some(px),
            _ => { None }
        }
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
                let width = Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?;
                self.declarations.push(PropertyDeclaration::BorderBottomWidth(width));
                self.declarations.push(PropertyDeclaration::BorderTopWidth(width));
                self.declarations.push(PropertyDeclaration::BorderRightWidth(width));
                self.declarations.push(PropertyDeclaration::BorderLeftWidth(width));
                Ok(())
            },
            "border-bottom-width" => { self.declarations.push(PropertyDeclaration::BorderBottomWidth(Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-top-width" => { self.declarations.push(PropertyDeclaration::BorderTopWidth(Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-left-width" => { self.declarations.push(PropertyDeclaration::BorderLeftWidth(Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-right-width" => { self.declarations.push(PropertyDeclaration::BorderRightWidth(Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "border-radius" => { self.declarations.push(PropertyDeclaration::BorderRadius(Length::parse(parser)?.to_px().ok_or_else(|| parser.new_custom_error(PropertyParseErrorKind::UnsupportedUnit))?)); Ok(()) },
            "background-color" => {
                self.declarations.push(PropertyDeclaration::Color(CssColor::parse(parser)?.to_color()));
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
