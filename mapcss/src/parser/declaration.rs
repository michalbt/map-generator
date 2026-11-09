use cssparser::color::{OPAQUE, clamp_unit_f32, parse_hash_color, parse_named_color};
use cssparser::{Parser, Token, ascii_case_insensitive_map};

use crate::error::{MapCssErrorKind, MapCssParseError};
use crate::parser::ParseContext;
use crate::parser::parse::Parse;
use crate::types::declaration::{
    AbsoluteFloatOrRelativePercentage, Color, Declaration, Float, FontStyle, FontVariant,
    FontWeight, Integer, IntegerList, Percentage, TextDecoration, TextPosition, TextTransform, Url,
};

impl Parse for Integer {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        input
            .try_parse(|inp| inp.expect_integer())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::IntegerExpected))
    }
}

impl Parse for Float {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        input
            .try_parse(|inp| inp.expect_number())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::FloatExpected))
    }
}

impl Parse for Percentage {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        input
            .try_parse(|inp| inp.expect_percentage())
            .map(Percentage)
            .map_err(|_| input.new_custom_error(MapCssErrorKind::PercentageExpected))
    }
}

impl Parse for AbsoluteFloatOrRelativePercentage {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(percentage) = input.try_parse(ctx.parse::<Percentage>()) {
            Ok(Self::RelativePercentage(percentage))
        } else if let Ok(float) = input.try_parse(ctx.parse::<Float>()) {
            Ok(Self::AbsoluteFloat(float))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::FloatOrPercentageExpected))
        }
    }
}

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
    if !(0.0..=1.0).contains(&value) {
        Err(input.new_custom_error(MapCssErrorKind::ColorUnitFloatOutOfRange))
    } else {
        Ok(clamp_unit_f32(value))
    }
}

fn parse_alpha<'i>(input: &mut Parser<'i, '_>) -> Result<f32, MapCssParseError<'i>> {
    let value = input
        .try_parse(|inp| inp.expect_number())
        .map_err(|_| input.new_custom_error(MapCssErrorKind::ColorAlphaUnitFloatExpected))?;
    if !(0.0..=1.0).contains(&value) {
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
) -> Result<Color, MapCssParseError<'i>> {
    let r = parse_color_unit_float(input)?;
    parse_comma(input)?;
    let g = parse_color_unit_float(input)?;
    parse_comma(input)?;
    let b = parse_color_unit_float(input)?;
    Ok(Color { r, g, b, a: OPAQUE })
}

fn parse_rgba_function_content<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Color, MapCssParseError<'i>> {
    let Color { r, g, b, a: _ } = parse_rgb_function_content(input)?;
    parse_comma(input)?;
    let a = parse_alpha(input)?;
    Ok(Color { r, g, b, a })
}

impl Parse for Color {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(hash_string) = input.try_parse(parse_hash_token) {
            parse_hash_color(hash_string.as_bytes())
                .map(|(r, g, b, a)| Color { r, g, b, a })
                .map_err(|_| input.new_custom_error(MapCssErrorKind::InvalidHashColor))
        } else if input
            .try_parse(|inp| inp.expect_function_matching("rgb"))
            .is_ok()
        {
            input.parse_nested_block(parse_rgb_function_content)
        } else if input
            .try_parse(|inp| inp.expect_function_matching("rgba"))
            .is_ok()
        {
            input.parse_nested_block(parse_rgba_function_content)
        } else if let Ok(color_name) =
            input.try_parse(|inp| inp.expect_ident().map(ToString::to_string))
        {
            parse_named_color(&color_name)
                .map(|(r, g, b)| Color { r, g, b, a: OPAQUE })
                .map_err(|_| input.new_custom_error(MapCssErrorKind::InvalidNamedColor))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::ColorExpected))
        }
    }
}

impl Parse for Url {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        input
            .try_parse(|inp| inp.expect_url())
            .map(|s| Url(s.to_string()))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::UrlExpected))
    }
}

/// Expects the list to have at least 2 elements.
impl Parse for IntegerList {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let first = input.try_parse(|inp| inp.expect_integer()).map_err(|_| {
            input.new_custom_error(MapCssErrorKind::CommaSeparatedIntegerListExpected)
        })?;
        input.try_parse(parse_comma).map_err(|_| {
            input.new_custom_error(MapCssErrorKind::CommaSeparatedIntegerListTooShort)
        })?;
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
}

impl Parse for FontWeight {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let identifier = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::FontWeightExpected))?;
        ascii_case_insensitive_map! {
            FontWeightMap -> FontWeight = {
                "normal" => FontWeight::Normal,
                "bold" => FontWeight::Bold,
            }
        }
        FontWeightMap::get(&identifier)
            .map(Clone::clone)
            .ok_or(input.new_custom_error(MapCssErrorKind::InvalidFontWeight))
    }
}

impl Parse for FontStyle {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let identifier = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::FontStyleExpected))?;
        ascii_case_insensitive_map! {
            FontStyleMap -> FontStyle = {
                "normal" => FontStyle::Normal,
                "italic" => FontStyle::Italic,
            }
        }
        FontStyleMap::get(&identifier)
            .map(Clone::clone)
            .ok_or(input.new_custom_error(MapCssErrorKind::InvalidFontStyle))
    }
}

impl Parse for FontVariant {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let identifier = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::FontVariantExpected))?;
        ascii_case_insensitive_map! {
            FontVariantMap -> FontVariant = {
                "normal" => FontVariant::Normal,
                "small-caps" => FontVariant::SmallCaps,
            }
        }
        FontVariantMap::get(&identifier)
            .map(Clone::clone)
            .ok_or(input.new_custom_error(MapCssErrorKind::InvalidFontVariant))
    }
}

impl Parse for TextDecoration {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let identifier = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::TextDecorationExpected))?;
        ascii_case_insensitive_map! {
            TextDecorationMap -> TextDecoration = {
                "none" => TextDecoration::None,
                "underline" => TextDecoration::Underline,
            }
        }
        TextDecorationMap::get(&identifier)
            .map(Clone::clone)
            .ok_or(input.new_custom_error(MapCssErrorKind::InvalidTextDecoration))
    }
}

impl Parse for TextTransform {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let identifier = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::TextTransformExpected))?;
        ascii_case_insensitive_map! {
            TextTransformMap -> TextTransform = {
                "none" => TextTransform::None,
                "uppercase" => TextTransform::Uppercase,
                "lowercase" => TextTransform::Lowercase,
                "capitalize" => TextTransform::Capitalize,
            }
        }
        TextTransformMap::get(&identifier)
            .map(Clone::clone)
            .ok_or(input.new_custom_error(MapCssErrorKind::InvalidTextTransform))
    }
}

impl Parse for TextPosition {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        _ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let identifier = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::TextPositionExpected))?;
        ascii_case_insensitive_map! {
            TextPositionMap -> TextPosition = {
                "line" => TextPosition::Line,
                "center" => TextPosition::Center,
            }
        }
        TextPositionMap::get(&identifier)
            .map(Clone::clone)
            .ok_or(input.new_custom_error(MapCssErrorKind::InvalidTextPosition))
    }
}

impl Parse for Declaration {
    fn parse<'i>(
        input: &mut Parser<'i, '_>,
        ctx: &mut ParseContext,
    ) -> Result<Self, MapCssParseError<'i>> {
        let property_name = input
            .try_parse(|inp| inp.expect_ident().map(ToString::to_string))
            .map_err(|_| input.new_custom_error(MapCssErrorKind::PropertyNameExpected))?;
        input
            .try_parse(|inp| inp.expect_colon())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::ColonExpected))?;
        let property_value_parser = Declaration::get_value_parser(&property_name)
            .ok_or(input.new_custom_error(MapCssErrorKind::UnsupportedPropertyName))?;
        let declaration = property_value_parser(input, ctx)?;
        input
            .try_parse(|inp| inp.expect_semicolon())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::SemicolonExpected))?;
        Ok(declaration)
    }
}
