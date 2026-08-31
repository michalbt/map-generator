use cssparser::{Parser, ascii_case_insensitive_map};

use crate::{
    error::{MapCssErrorKind, MapCssParseError},
    parser::Parse,
    types::selector::{
        BasicSelector, ObjectType, Regex, SelectorAlternatives, SelectorChain, TagKey,
        TagNumericValue, TagStringValue, TagValue, Test,
    },
};

fn parse_non_special_object_type(input: &mut Parser) -> Result<ObjectType, ()> {
    if let Ok(identifier) = input.expect_ident() {
        ascii_case_insensitive_map! {
            ObjectTypeMap -> ObjectType = {
                "node" => ObjectType::Node,
                "way" => ObjectType::Way,
                "relation" => ObjectType::Relation,
                "area" => ObjectType::Area,
                "line" => ObjectType::Line,
                "canvas" => ObjectType::Canvas,
            }
        }
        if let Some(object_type) = ObjectTypeMap::get(&**identifier) {
            Ok(*object_type)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

impl Parse for ObjectType {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(object_type) = input.try_parse(parse_non_special_object_type) {
            Ok(object_type)
        } else if let Ok(()) = input.try_parse(|inp| inp.expect_delim('*')) {
            Ok(ObjectType::Any)
        } else {
            Err(input.new_custom_error(MapCssErrorKind::ObjectTypeExpected))
        }
    }
}

impl Parse for TagKey {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(string) =
            input.try_parse(|inp| inp.expect_ident_or_string().map(|s| TagKey(s.to_string())))
        {
            Ok(string)
        } else {
            Err(input.new_custom_error(MapCssErrorKind::TagKeyExpected))
        }
    }
}

impl Parse for TagNumericValue {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(integer) = input.try_parse(|inp| inp.expect_integer()) {
            Ok(Self::Integer(integer))
        } else if let Ok(float) = input.try_parse(|inp| inp.expect_number()) {
            Ok(Self::Float(float))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::TagNumericValueExpected))
        }
    }
}

impl Parse for TagStringValue {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(string) = input.try_parse(|inp| {
            inp.expect_ident_or_string()
                .map(|s| TagStringValue(s.to_string()))
        }) {
            Ok(string)
        } else {
            Err(input.new_custom_error(MapCssErrorKind::TagStringValueExpected))
        }
    }
}

impl Parse for TagValue {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(numeric) = input.try_parse(TagNumericValue::parse) {
            Ok(Self::Numeric(numeric))
        } else if let Ok(string) = input.try_parse(TagStringValue::parse) {
            Ok(Self::String(string))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::TagValueExpected))
        }
    }
}

impl Parse for Regex {
    /// Only quoted regexes are supported
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(string) = input.try_parse(|inp| inp.expect_string().map(ToString::to_string)) {
            regex::Regex::new(&string)
                .map(Regex)
                .map_err(|_| input.new_custom_error(MapCssErrorKind::InvalidRegex))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::RegexExpected))
        }
    }
}

fn parse_equal_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('=').map_err(|_| ())
}

fn parse_not_equal_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('!').map_err(|_| ())?;
    input.expect_delim('=').map_err(|_| ())
}

fn parse_less_than_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('<').map_err(|_| ())
}

fn parse_less_than_or_equal_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('<').map_err(|_| ())?;
    input.expect_delim('=').map_err(|_| ())
}

fn parse_greater_than_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('>').map_err(|_| ())
}

fn parse_greater_than_or_equal_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('>').map_err(|_| ())?;
    input.expect_delim('=').map_err(|_| ())
}

fn parse_matches_operator(input: &mut Parser) -> Result<(), ()> {
    input.expect_delim('=').map_err(|_| ())?;
    input.expect_delim('~').map_err(|_| ())
}

fn parse_test_contents<'i>(input: &mut Parser<'i, '_>) -> Result<Test, MapCssParseError<'i>> {
    if let Ok(()) = input.try_parse(|inp| inp.expect_delim('!')) {
        TagKey::parse(input).map(Test::NotSet)
    } else if let Ok(tag_key) = input.try_parse(TagKey::parse) {
        // Order is important: operators must be before their prefixes ('=~' before '=' etc.)
        if let Ok(()) = input.try_parse(parse_matches_operator) {
            Regex::parse(input).map(move |regex| Test::Matches(tag_key, regex))
        } else if let Ok(()) = input.try_parse(parse_less_than_or_equal_operator) {
            TagNumericValue::parse(input).map(move |value| Test::LessThanOrEqual(tag_key, value))
        } else if let Ok(()) = input.try_parse(parse_greater_than_or_equal_operator) {
            TagNumericValue::parse(input).map(move |value| Test::GreaterThanOrEqual(tag_key, value))
        } else if let Ok(()) = input.try_parse(parse_not_equal_operator) {
            TagValue::parse(input).map(move |value| Test::NotEqual(tag_key, value))
        } else if let Ok(()) = input.try_parse(parse_less_than_operator) {
            TagNumericValue::parse(input).map(move |value| Test::LessThan(tag_key, value))
        } else if let Ok(()) = input.try_parse(parse_greater_than_operator) {
            TagNumericValue::parse(input).map(move |value| Test::GreaterThan(tag_key, value))
        } else if let Ok(()) = input.try_parse(parse_equal_operator) {
            TagValue::parse(input).map(move |value| Test::Equal(tag_key, value))
        } else if let Ok(()) = input.try_parse(|inp| inp.expect_exhausted()) {
            Ok(Test::Set(tag_key))
        } else {
            Err(input.new_custom_error(MapCssErrorKind::InvalidTokenInsideTest))
        }
    } else {
        Err(input.new_custom_error(MapCssErrorKind::TagKeyOrExclamationExpected))
    }
}

impl Parse for Test {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        input
            .try_parse(|inp| inp.expect_square_bracket_block())
            .map_err(|_| input.new_custom_error(MapCssErrorKind::TestBlockExpected))?;
        input.parse_nested_block(parse_test_contents)
    }
}

impl Parse for BasicSelector {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let object_type = ObjectType::parse(input)?;
        let mut tests = vec![];
        while let Ok(test) = Test::parse(input) {
            tests.push(test);
        }
        Ok(BasicSelector::new(object_type, tests))
    }
}

impl Parse for SelectorChain {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let mut chain = vec![];
        let first_selector = BasicSelector::parse(input)?;
        chain.push(first_selector);
        while let Ok(selector) = BasicSelector::parse(input) {
            chain.push(selector);
        }
        Ok(SelectorChain::new(chain))
    }
}

impl Parse for SelectorAlternatives {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let mut alternatives = vec![];
        let first_chain = SelectorChain::parse(input)?;
        alternatives.push(first_chain);
        while let Ok(()) = input.expect_comma() {
            if let Ok(chain) = input.try_parse(SelectorChain::parse) {
                alternatives.push(chain);
            } else {
                return Err(input.new_custom_error(MapCssErrorKind::SelectorChainExpected));
            }
        }
        Ok(SelectorAlternatives::new(alternatives))
    }
}
