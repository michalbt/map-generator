use cssparser::{
    Parser, Token,
    color::{OPAQUE, clamp_unit_f32, parse_hash_color},
};

use crate::{
    error::{MapCssErrorKind, MapCssParseError},
    parser::Parse,
    types::stylesheet::{Declaration, PropertyValue, Rule, Stylesheet},
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

impl Parse for PropertyValue {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        todo!()
    }
}

impl Parse for Declaration {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        todo!()
    }
}

impl Parse for Rule {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        todo!()
    }
}

impl Parse for Stylesheet {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        todo!()
    }
}
