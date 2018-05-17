use cssparser::{Parser,ParserInput,DeclarationListParser,DeclarationParser,RuleListParser,QualifiedRuleParser,CowRcStr,ParseError,AtRuleParser};
use failure::Error;

use super::style::*;

/// A CSS declaration
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
}


/// A CSS selector.
pub struct Selector
{
    /// TODO
    class: String
}

/// A CSS rule-set.
pub struct Rule
{
    /// Selector.
    selector: Selector,
    /// List of CSS declarations.
    declarations: Vec<PropertyDeclaration>
}

/// A stylesheet.
pub struct Stylesheet
{
    /// List of rule-sets.
    rules: Vec<Rule>
}

pub struct RulesParser;

#[derive(Debug)]
pub enum RuleParseErrorKind {
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
        let mut decl_list_parser = DeclarationListParser::new(parser, PropertyDeclarationParser);
        let mut rule = Rule {
            declarations: Vec::new(),
            selector: prelude,
        };

        while let Some(result) = decl_list_parser.next() {
            match result {
                Ok(decl) => {
                    // Got a decl
                    rule.declarations.push(decl);
                }
                Err(e) => {
                    // FIXME error reporting
                    warn!("CSS parse error: {:?}", e);
                }
            }
        }

        Ok(rule)
    }
}

pub struct PropertyDeclarationParser;

impl<'i> AtRuleParser<'i> for PropertyDeclarationParser {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = PropertyDeclaration;
    type Error = PropertyParseErrorKind<'i>;
}

#[derive(Debug)]
pub enum PropertyParseErrorKind<'i> {
    UnknownProperty(CowRcStr<'i>),
    Other
}

impl<'i> DeclarationParser<'i> for PropertyDeclarationParser {
    type Declaration = PropertyDeclaration;
    type Error = PropertyParseErrorKind<'i>;

    fn parse_value<'t>(&mut self, name: CowRcStr<'i>, parser: &mut Parser<'i, 't>)
                       -> Result<Self::Declaration, ParseError<'i, Self::Error>>
    {
        match name.as_ref() {
            "color" => { Ok(PropertyDeclaration::Color((0.0,0.0,0.0,0.0))) },
            "background-color" => { Ok(PropertyDeclaration::Color((0.0,0.0,0.0,0.0))) },
            _ => Err(parser.new_custom_error(PropertyParseErrorKind::UnknownProperty(name)))
        }
    }
}

/// Parse a stylesheet.
fn parse_stylesheet(text: &str) -> Result<Stylesheet, Error>
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
                // TODO
            }
        }
    }

    unimplemented!()
}
