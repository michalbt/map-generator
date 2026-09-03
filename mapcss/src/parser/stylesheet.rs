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
