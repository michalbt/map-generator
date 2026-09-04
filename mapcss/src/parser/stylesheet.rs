use cssparser::{
    Delimiter, ParseErrorKind, Parser, Token,
    color::{OPAQUE, clamp_unit_f32, parse_hash_color},
};

use crate::{
    error::{MapCssErrorKind, MapCssParseError},
    parser::Parse,
    types::{
        selector::SelectorAlternatives,
        stylesheet::{Declaration, PropertyValue, Rule, Stylesheet},
    },
};

fn parse_hash_token(input: &mut Parser) -> Result<String, ()> {
    match input.next().map_err(|_| ())? {
        Token::IDHash(s) | Token::Hash(s) => Ok(s.to_string()),
        _ => Err(()),
    }
}

fn parse_color_unit_float<'i>(input: &mut Parser<'i, '_>) -> Result<u8, MapCssParseError<'i>> {
    let value = input
        .try_parse(|inp| inp.expect_number())
        .map_err(|_| input.new_custom_error(MapCssErrorKind::ColorUnitFloatExpected))?;
    if value < 0.0 || value > 1.0 {
        Err(input.new_custom_error(MapCssErrorKind::ColorUnitFloatOutOfRange))
    } else {
        Ok(clamp_unit_f32(value))
    }
}

fn parse_alpha<'i>(input: &mut Parser<'i, '_>) -> Result<f32, MapCssParseError<'i>> {
    let value = input
        .try_parse(|inp| inp.expect_number())
        .map_err(|_| input.new_custom_error(MapCssErrorKind::ColorAlphaUnitFloatExpected))?;
    if value < 0.0 || value > 1.0 {
        Err(input.new_custom_error(MapCssErrorKind::ColorAlphaUnitFloatOutOfRange))
    } else {
        Ok(value)
    }
}

fn parse_comma<'i>(input: &mut Parser<'i, '_>) -> Result<(), MapCssParseError<'i>> {
    input
        .try_parse(|inp| inp.expect_comma())
        .map_err(|_| input.new_custom_error(MapCssErrorKind::CommaExpected))
}

fn parse_rgb_function_content<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(u8, u8, u8), MapCssParseError<'i>> {
    let r = parse_color_unit_float(input)?;
    parse_comma(input)?;
    let g = parse_color_unit_float(input)?;
    parse_comma(input)?;
    let b = parse_color_unit_float(input)?;
    Ok((r, g, b))
}

fn parse_rgba_function_content<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(u8, u8, u8, f32), MapCssParseError<'i>> {
    let (r, g, b) = parse_rgb_function_content(input)?;
    parse_comma(input)?;
    let alpha = parse_alpha(input)?;
    Ok((r, g, b, alpha))
}

fn parse_color<'i>(input: &mut Parser<'i, '_>) -> Result<(u8, u8, u8, f32), MapCssParseError<'i>> {
    if let Ok(hash_string) = input.try_parse(parse_hash_token) {
        parse_hash_color(hash_string.as_bytes())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::InvalidHashColor))
    } else if input
        .try_parse(|inp| inp.expect_function_matching("rgb"))
        .is_ok()
    {
        input
            .parse_nested_block(parse_rgb_function_content)
            .map(|(r, g, b)| (r, g, b, OPAQUE))
    } else if input
        .try_parse(|inp| inp.expect_function_matching("rgba"))
        .is_ok()
    {
        input.parse_nested_block(parse_rgba_function_content)
    } else {
        Err(input.new_custom_error(MapCssErrorKind::ColorExpected))
    }
}

/// Parses a comma-separated list with at least 2 elements.
fn parse_comma_separated_integer_list<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<i32>, MapCssParseError<'i>> {
    let first = input
        .try_parse(|inp| inp.expect_integer())
        .map_err(|_| input.new_custom_error(MapCssErrorKind::CommaSeparatedIntegerListExpected))?;
    input
        .try_parse(parse_comma)
        .map_err(|_| input.new_custom_error(MapCssErrorKind::CommaSeparatedIntegerListTooShort))?;
    let second = input
        .try_parse(|inp| inp.expect_integer())
        .map_err(|_| input.new_custom_error(MapCssErrorKind::IntegerExpected))?;
    let mut list = vec![first, second];

    while let Ok(()) = parse_comma(input) {
        let integer = input
            .try_parse(|inp| inp.expect_integer())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::IntegerExpected))?;
        list.push(integer);
    }
    Ok(list)
}

impl Parse for PropertyValue {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        match input.try_parse(parse_comma_separated_integer_list) {
            Ok(list) => return Ok(Self::IntegerList(list)),
            Err(e)
                if matches!(
                    e.kind,
                    ParseErrorKind::Custom(
                        MapCssErrorKind::CommaSeparatedIntegerListExpected
                            | MapCssErrorKind::CommaSeparatedIntegerListTooShort
                    )
                ) => {} // continue with a different parser
            Err(e) => return Err(e),
        }
        match input.try_parse(parse_color) {
            Ok((r, g, b, a)) => return Ok(Self::Color(r, g, b, a)),
            Err(e)
                if matches!(
                    e.kind,
                    ParseErrorKind::Custom(MapCssErrorKind::ColorExpected)
                ) => {} // continue with a different parser
            Err(e) => return Err(e),
        }
        if let Ok(percentage) = input.try_parse(|inp| inp.expect_percentage()) {
            Ok(Self::Percentage(percentage))
        } else if let Ok(url) = input.try_parse(|inp| inp.expect_url()) {
            Ok(Self::Url(url.to_string()))
        } else if let Ok(integer) = input.try_parse(|inp| inp.expect_integer()) {
            Ok(Self::Integer(integer))
        } else if let Ok(float) = input.try_parse(|inp| inp.expect_number()) {
            Ok(Self::Float(float))
        } else if let Ok(string) =
            input.try_parse(|inp| inp.expect_ident_or_string().map(ToString::to_string))
        {
            Ok(Self::String(string))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::PropertyValueExpected))
        }
    }
}

impl Parse for Declaration {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let property_name = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::PropertyNameExpected))?;
        input
            .try_parse(|inp| inp.expect_colon())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::ColonExpected))?;
        let property_value = PropertyValue::parse(input)?;
        input
            .try_parse(|inp| inp.expect_semicolon())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::SemicolonExpected))?;
        Ok(Self {
            property_name,
            property_value,
        })
    }
}

fn parse_declaration_block_content<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<Declaration>, MapCssParseError<'i>> {
    let mut declarations = vec![];
    while !input.is_exhausted() {
        let declaration = Declaration::parse(input)?;
        declarations.push(declaration);
    }
    Ok(declarations)
}

impl Parse for Rule {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let selector =
            input.parse_until_before(Delimiter::CurlyBracketBlock, SelectorAlternatives::parse)?;
        input
            .try_parse(|inp| inp.expect_curly_bracket_block())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::DeclarationBlockExpected))?;
        let declarations = input.parse_nested_block(parse_declaration_block_content)?;
        Ok(Self {
            selector,
            declarations,
        })
    }
}

impl Parse for Stylesheet {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let mut rules = vec![];
        while !input.is_exhausted() {
            let rule = Rule::parse(input)?;
            rules.push(rule);
        }
        Ok(Self { rules })
    }
}

#[cfg(test)]
mod tests {
    use cssparser::BasicParseErrorKind;

    use crate::{
        parser::test_helpers::*,
        types::selector::{
            BasicSelector, ObjectType, SelectorChain, TagKey, TagStringValue, TagValue, Test,
        },
    };

    use super::*;

    #[test]
    fn rgb_function_content_parsing() {
        parse_all_and_expect(parse_rgb_function_content, "0.0, 0.5, 1.0", (0, 128, 255));
        parse_all_and_expect(parse_rgb_function_content, "1,1,1", (255, 255, 255));
        parse_all_and_expect_error(
            parse_rgb_function_content,
            "255, 255, 0",
            MapCssErrorKind::ColorUnitFloatOutOfRange,
        );
        parse_all_and_expect_error(
            parse_rgb_function_content,
            "1, 1, ",
            MapCssErrorKind::ColorUnitFloatExpected,
        );
        parse_all_and_expect_error(
            parse_rgb_function_content,
            "1, 1",
            MapCssErrorKind::CommaExpected,
        );
        parse_all_and_expect_error(
            parse_rgb_function_content,
            "0.5, b",
            MapCssErrorKind::ColorUnitFloatExpected,
        );
        parse_all_and_expect_remaining(parse_rgb_function_content, "0, 0, 0, 0", ", 0");
    }

    #[test]
    fn rgba_function_content_parsing() {
        parse_all_and_expect(
            parse_rgba_function_content,
            "1.0, 0.5, 0.0, 0.7",
            (255, 128, 0, 0.7),
        );
        parse_all_and_expect(parse_rgba_function_content, "0,0,0,1", (0, 0, 0, 1.0));
        parse_all_and_expect_error(
            parse_rgba_function_content,
            "0, 0, 0, 2",
            MapCssErrorKind::ColorAlphaUnitFloatOutOfRange,
        );
        parse_all_and_expect_error(
            parse_rgba_function_content,
            "0, 0, 0,",
            MapCssErrorKind::ColorAlphaUnitFloatExpected,
        );
        parse_all_and_expect_error(
            parse_rgba_function_content,
            "0, 0, 0",
            MapCssErrorKind::CommaExpected,
        );
    }

    #[test]
    fn color_parsing() {
        parse_all_and_expect(parse_color, "rgb(0.5, 0.5, 0.5)", (128, 128, 128, 1.0));
        parse_all_and_expect(parse_color, "rgba(0, 0.5, 1, 0.5)", (0, 128, 255, 0.5));
        parse_all_and_expect(parse_color, "#abcdef", (0xab, 0xcd, 0xef, 1.0));
        parse_all_and_expect(parse_color, "#123", (0x11, 0x22, 0x33, 1.0));
        parse_all_and_expect(parse_color, "#A1B2", (0xaa, 0x11, 0xbb, 2.0 / 15.0));
        parse_all_and_expect(
            parse_color,
            "#87654321",
            (0x87, 0x65, 0x43, 0x21 as f32 / 0xff as f32),
        );
        parse_all_and_expect_error(parse_color, "red", MapCssErrorKind::ColorExpected);
        parse_all_and_expect_basic_error(
            parse_color,
            "rgb(0, 0, 0, 0)",
            BasicParseErrorKind::UnexpectedToken(Token::Comma),
        );
        parse_all_and_expect_basic_error(
            parse_color,
            "rgba(1, 1, 1, 1 something)",
            BasicParseErrorKind::UnexpectedToken(Token::Ident("something".into())),
        );
        parse_all_and_expect_error(parse_color, "#12345", MapCssErrorKind::InvalidHashColor);
        parse_all_and_expect_error(parse_color, "#123456789", MapCssErrorKind::InvalidHashColor);
    }

    #[test]
    fn comma_separated_integer_list_parsing() {
        parse_all_and_expect(parse_comma_separated_integer_list, "1, 2", vec![1, 2]);
        parse_all_and_expect(
            parse_comma_separated_integer_list,
            "-4, 15, 0",
            vec![-4, 15, 0],
        );
        parse_all_and_expect_error(
            parse_comma_separated_integer_list,
            "1",
            MapCssErrorKind::CommaSeparatedIntegerListTooShort,
        );
        parse_all_and_expect_error(
            parse_comma_separated_integer_list,
            "1,",
            MapCssErrorKind::IntegerExpected,
        );
        parse_all_and_expect_error(
            parse_comma_separated_integer_list,
            "1, 2,",
            MapCssErrorKind::IntegerExpected,
        );
        parse_all_and_expect_error(
            parse_comma_separated_integer_list,
            "1, 2, 1.0",
            MapCssErrorKind::IntegerExpected,
        );
        parse_all_and_expect_error(
            parse_comma_separated_integer_list,
            "something",
            MapCssErrorKind::CommaSeparatedIntegerListExpected,
        );
    }

    #[test]
    fn property_value_parsing() {
        parse_all_and_expect(
            PropertyValue::parse,
            "red",
            PropertyValue::String("red".into()),
        );
        parse_all_and_expect(
            PropertyValue::parse,
            "\":-)\"",
            PropertyValue::String(":-)".into()),
        );
        parse_all_and_expect(PropertyValue::parse, "-5", PropertyValue::Integer(-5));
        parse_all_and_expect(PropertyValue::parse, "7.3", PropertyValue::Float(7.3));
        parse_all_and_expect(PropertyValue::parse, "12%", PropertyValue::Percentage(0.12));
        parse_all_and_expect(
            PropertyValue::parse,
            "#ffffff",
            PropertyValue::Color(255, 255, 255, 1.0),
        );
        parse_all_and_expect(
            PropertyValue::parse,
            "rgb(0, 0, 0)",
            PropertyValue::Color(0, 0, 0, 1.0),
        );
        parse_all_and_expect(
            PropertyValue::parse,
            "rgba(1, 1, 1, 0.5)",
            PropertyValue::Color(255, 255, 255, 0.5),
        );
        parse_all_and_expect(
            PropertyValue::parse,
            "url(/icons/icon.svg)",
            PropertyValue::Url("/icons/icon.svg".into()),
        );
        parse_all_and_expect(
            PropertyValue::parse,
            "url(\"https://example.com/icon\")",
            PropertyValue::Url("https://example.com/icon".into()),
        );
        parse_all_and_expect(
            PropertyValue::parse,
            "1, 2, 3",
            PropertyValue::IntegerList(vec![1, 2, 3]),
        );
        parse_all_and_expect_error(
            PropertyValue::parse,
            "rgb()",
            MapCssErrorKind::ColorUnitFloatExpected,
        );
        parse_all_and_expect_error(
            PropertyValue::parse,
            "1, 2,",
            MapCssErrorKind::IntegerExpected,
        );
        parse_all_and_expect_error(
            PropertyValue::parse,
            "uurl(something.svg)",
            MapCssErrorKind::PropertyValueExpected,
        );
    }

    #[test]
    fn declaration_parsing() {
        parse_all_and_expect(
            Declaration::parse,
            "name: value;",
            Declaration {
                property_name: "name".into(),
                property_value: PropertyValue::String("value".into()),
            },
        );
        parse_all_and_expect(
            Declaration::parse,
            "name-with-dashes: 1, 2, 3;",
            Declaration {
                property_name: "name-with-dashes".into(),
                property_value: PropertyValue::IntegerList(vec![1, 2, 3]),
            },
        );
        parse_all_and_expect(
            Declaration::parse,
            "name: \"a;b\";",
            Declaration {
                property_name: "name".into(),
                property_value: PropertyValue::String("a;b".into()),
            },
        );
        parse_all_and_expect_error(
            Declaration::parse,
            "name with spaces: value;",
            MapCssErrorKind::ColonExpected,
        );
        parse_all_and_expect_error(
            Declaration::parse,
            "123name: value;",
            MapCssErrorKind::PropertyNameExpected,
        );
        parse_all_and_expect_error(
            Declaration::parse,
            "name: ;",
            MapCssErrorKind::PropertyValueExpected,
        );
        parse_all_and_expect_error(
            Declaration::parse,
            "name: rgb(-5);",
            MapCssErrorKind::ColorUnitFloatOutOfRange,
        );
        parse_all_and_expect_error(
            Declaration::parse,
            "name: value",
            MapCssErrorKind::SemicolonExpected,
        );
        parse_all_and_expect_error(
            Declaration::parse,
            "name: value1 value2;",
            MapCssErrorKind::SemicolonExpected,
        );
        parse_all_and_expect_basic_error(
            Declaration::parse,
            "name: rgb(0, 0, 0;",
            BasicParseErrorKind::UnexpectedToken(Token::Semicolon),
        );
        parse_all_and_expect_remaining(
            Declaration::parse,
            "name1: value1; name2: value2;",
            " name2: value2;",
        );
    }

    #[test]
    fn rule_parsing() {
        parse_all_and_expect(
            Rule::parse,
            "node { name1: value1; name2: value2; }",
            Rule {
                selector: SelectorAlternatives::new(vec![SelectorChain::new(vec![
                    BasicSelector::new(ObjectType::Node, vec![]),
                ])]),
                declarations: vec![
                    Declaration {
                        property_name: "name1".into(),
                        property_value: PropertyValue::String("value1".into()),
                    },
                    Declaration {
                        property_name: "name2".into(),
                        property_value: PropertyValue::String("value2".into()),
                    },
                ],
            },
        );
        parse_all_and_expect(
            Rule::parse,
            "node, way [key=value] {}",
            Rule {
                selector: SelectorAlternatives::new(vec![
                    SelectorChain::new(vec![BasicSelector::new(ObjectType::Node, vec![])]),
                    SelectorChain::new(vec![BasicSelector::new(
                        ObjectType::Way,
                        vec![Test::Equal(
                            TagKey("key".into()),
                            TagValue::String(TagStringValue("value".into())),
                        )],
                    )]),
                ]),
                declarations: vec![],
            },
        );
        parse_all_and_expect_error(
            Rule::parse,
            "* { name: value }",
            MapCssErrorKind::SemicolonExpected,
        );
        parse_all_and_expect_error(
            Rule::parse,
            "node [key=] {}",
            MapCssErrorKind::TagValueExpected,
        );
        parse_all_and_expect_error(
            Rule::parse,
            "node",
            MapCssErrorKind::DeclarationBlockExpected,
        );
        // This should be invalid but I found no way to recognize it with cssparser
        parse_all_and_expect(
            Rule::parse,
            "node {",
            Rule {
                selector: SelectorAlternatives::new(vec![SelectorChain::new(vec![
                    BasicSelector::new(ObjectType::Node, vec![]),
                ])]),
                declarations: vec![],
            },
        );
        parse_all_and_expect_remaining(Rule::parse, "node {} way {}", " way {}");
    }

    #[test]
    fn stylesheet_parsing() {
        parse_all_and_expect(Stylesheet::parse, "", Stylesheet { rules: vec![] });
        parse_all_and_expect(
            Stylesheet::parse,
            "node { name1: value1; } way { name2: value2; }",
            Stylesheet {
                rules: vec![
                    Rule {
                        selector: SelectorAlternatives::new(vec![SelectorChain::new(vec![
                            BasicSelector::new(ObjectType::Node, vec![]),
                        ])]),
                        declarations: vec![Declaration {
                            property_name: "name1".into(),
                            property_value: PropertyValue::String("value1".into()),
                        }],
                    },
                    Rule {
                        selector: SelectorAlternatives::new(vec![SelectorChain::new(vec![
                            BasicSelector::new(ObjectType::Way, vec![]),
                        ])]),
                        declarations: vec![Declaration {
                            property_name: "name2".into(),
                            property_value: PropertyValue::String("value2".into()),
                        }],
                    },
                ],
            },
        );
        parse_all_and_expect_error(
            Stylesheet::parse,
            "node {} {}",
            MapCssErrorKind::ObjectTypeExpected,
        );
        parse_all_and_expect_error(
            Stylesheet::parse,
            "node {something}",
            MapCssErrorKind::ColonExpected,
        );
        parse_all_and_expect_error(
            Stylesheet::parse,
            "node { name1: value1; way { name2: value2; }",
            MapCssErrorKind::ColonExpected,
        );
    }
}
