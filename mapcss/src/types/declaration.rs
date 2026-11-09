use crate::{
    error::MapCssParseError,
    parser::{Parse, ParseContext},
};

use cssparser::{Parser, ascii_case_insensitive_map};
use map_data::TagKey;

type PropertyValueParser =
    for<'i> fn(&mut Parser<'i, '_>, &mut ParseContext) -> Result<Declaration, MapCssParseError<'i>>;

macro_rules! typed_declaration_properties {
    ($(($name:literal, $enum_key:ident, $type:ty)),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) enum Declaration {
            $(
                $enum_key($type),
            )*
        }

        impl Declaration {
            pub fn get_value_parser(property_name: &str) -> Option<PropertyValueParser> {
                ascii_case_insensitive_map! {
                    PropertyValueParserMap -> PropertyValueParser = {
                        $(
                            $name => |input, ctx| <$type as Parse>::parse(input, ctx).map(Declaration::$enum_key),
                        )*
                    }
                }
                PropertyValueParserMap::get(property_name).copied()
            }
        }
    };
}

typed_declaration_properties!(
    ("fill-color", FillColor, Color),
    ("fill-opacity", FillOpacity, Float),
    ("fill-image", FillImage, Url),
    ("z-index", ZIndex, Integer),
    ("width", Width, Float),
    ("color", Color, Color),
    ("opacity", Opacity, Float),
    ("dashes", Dashes, IntegerList),
    ("casing-width", CasingWidth, Float),
    ("casing-color", CasingColor, Color),
    ("casing-opacity", CasingOpacity, Float),
    ("casing-dashes", CasingDashes, IntegerList),
    ("icon-opacity", IconOpacity, Float),
    ("icon-image", IconImage, Url),
    ("icon-width", IconWidth, AbsoluteFloatOrRelativePercentage),
    ("icon-height", IconHeight, AbsoluteFloatOrRelativePercentage),
    ("font-size", FontSize, Integer),
    ("max-width", MaxWidth, Float),
    ("font-weight", FontWeight, FontWeight),
    ("font-style", FontStyle, FontStyle),
    ("font-variant", FontVariant, FontVariant),
    ("text-decoration", TextDecoration, TextDecoration),
    ("text-transform", TextTransform, TextTransform),
    ("text", Text, TagKey),
    ("text-color", TextColor, Color),
    ("text-opacity", TextOpacity, Float),
    ("text-position", TextPosition, TextPosition),
    ("text-offset", TextOffset, Float),
);

pub(crate) type Integer = i32;

pub(crate) type Float = f32;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Percentage(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AbsoluteFloatOrRelativePercentage {
    AbsoluteFloat(Float),
    RelativePercentage(Percentage),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Url(pub String);

pub(crate) type IntegerList = Vec<Integer>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FontWeight {
    Normal,
    Bold,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FontStyle {
    Normal,
    Italic,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FontVariant {
    Normal,
    SmallCaps,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TextDecoration {
    None,
    Underline,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TextPosition {
    Line,
    Center,
}
