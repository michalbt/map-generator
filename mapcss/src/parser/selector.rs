use cssparser::{ParseErrorKind, Parser, ascii_case_insensitive_map};

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

fn create_regex_with_anchors(string: &str) -> Result<Regex, ()> {
    let add_start = if string.starts_with('^') { "" } else { "^" };
    let add_end = if string.ends_with('$') { "" } else { "$" };
    let full: String = add_start.to_owned() + string + add_end;
    regex::Regex::new(&full).map(Regex).map_err(|_| ())
}

impl Parse for Regex {
    /// Only quoted regexes are supported
    /// TODO: quote with slashes
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        if let Ok(string) = input.try_parse(|inp| inp.expect_string().map(ToString::to_string)) {
            create_regex_with_anchors(&string)
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

fn parse_test_content<'i>(input: &mut Parser<'i, '_>) -> Result<Test, MapCssParseError<'i>> {
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
        input.parse_nested_block(parse_test_content)
    }
}

impl Parse for BasicSelector {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let object_type = ObjectType::parse(input)?;
        let mut tests = vec![];
        loop {
            match Test::parse(input) {
                Ok(test) => tests.push(test),
                Err(e)
                    if matches!(
                        e.kind,
                        ParseErrorKind::Custom(MapCssErrorKind::TestBlockExpected)
                    ) =>
                {
                    // no more tests => stop parsing and return the results
                    return Ok(BasicSelector::new(object_type, tests));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl Parse for SelectorChain {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        let mut chain = vec![];
        let first_selector = BasicSelector::parse(input)?;
        chain.push(first_selector);
        while !input.is_exhausted() {
            let selector = BasicSelector::parse(input)?;
            chain.push(selector);
        }
        Ok(SelectorChain::new(chain))
    }
}

impl Parse for SelectorAlternatives {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, MapCssParseError<'i>> {
        input
            .parse_comma_separated(SelectorChain::parse)
            .map(SelectorAlternatives::new)
    }
}

#[cfg(test)]
mod tests {
    use cssparser::{BasicParseErrorKind, Token};

    use crate::parser::test_helpers::*;

    use super::*;

    #[test]
    fn object_type_parsing() {
        parse_all_and_expect(ObjectType::parse, "node", ObjectType::Node);
        parse_all_and_expect(ObjectType::parse, "Way", ObjectType::Way);
        parse_all_and_expect(ObjectType::parse, "RELATION", ObjectType::Relation);
        parse_all_and_expect(ObjectType::parse, "aReA", ObjectType::Area);
        parse_all_and_expect(ObjectType::parse, "line", ObjectType::Line);
        parse_all_and_expect(ObjectType::parse, "canvas", ObjectType::Canvas);
        parse_all_and_expect(ObjectType::parse, "*", ObjectType::Any);
        parse_all_and_expect_error(
            ObjectType::parse,
            "something",
            MapCssErrorKind::ObjectTypeExpected,
        );
        parse_all_and_expect_error(ObjectType::parse, "", MapCssErrorKind::ObjectTypeExpected);
    }

    #[test]
    fn tag_key_parsing() {
        parse_all_and_expect(TagKey::parse, "\"some key\"", TagKey("some key".into()));
        parse_all_and_expect(TagKey::parse, "\"123 + :#?\"", TagKey("123 + :#?".into()));
        parse_all_and_expect(TagKey::parse, "some_value", TagKey("some_value".into()));
        parse_all_and_expect_remaining(TagKey::parse, "key=value", "=value");
        parse_all_and_expect_error(TagKey::parse, "123", MapCssErrorKind::TagKeyExpected);
    }

    #[test]
    fn tag_value_parsing() {
        parse_all_and_expect(
            TagValue::parse,
            "\"some value\"",
            TagValue::String(TagStringValue("some value".into())),
        );
        parse_all_and_expect(
            TagValue::parse,
            "some_value",
            TagValue::String(TagStringValue("some_value".into())),
        );
        parse_all_and_expect_remaining(TagValue::parse, "some value", " value");
        parse_all_and_expect(
            TagValue::parse,
            "12.3",
            TagValue::Numeric(TagNumericValue::Float(12.3)),
        );
        parse_all_and_expect(
            TagValue::parse,
            "8765",
            TagValue::Numeric(TagNumericValue::Integer(8765)),
        );
        parse_all_and_expect_error(TagValue::parse, "123a", MapCssErrorKind::TagValueExpected);
    }

    #[test]
    fn regex_parsing() {
        parse_all_and_expect(
            Regex::parse,
            "\"\\\\d{4}-\\\\d{2}-\\\\d{2}\"", // Double escaping: one for Rust, one for MapCSS
            Regex(regex::Regex::new("^\\d{4}-\\d{2}-\\d{2}$").unwrap()),
        );
        parse_all_and_expect_error(Regex::parse, "/abc/", MapCssErrorKind::RegexExpected);
        parse_all_and_expect_error(Regex::parse, "abc", MapCssErrorKind::RegexExpected);
        parse_all_and_expect_error(Regex::parse, "\"(\"", MapCssErrorKind::InvalidRegex);
    }

    #[test]
    fn test_parsing() {
        parse_all_and_expect(
            Test::parse,
            "[key=value]",
            Test::Equal(
                TagKey("key".into()),
                TagValue::String(TagStringValue("value".into())),
            ),
        );
        parse_all_and_expect(
            Test::parse,
            "[\"key\"!=\"value\"]",
            Test::NotEqual(
                TagKey("key".into()),
                TagValue::String(TagStringValue("value".into())),
            ),
        );
        parse_all_and_expect(
            Test::parse,
            "[ \"key\" < 3 ]",
            Test::LessThan(TagKey("key".into()), TagNumericValue::Integer(3)),
        );
        parse_all_and_expect(
            Test::parse,
            "[\"#key 123?\"<=2.7]",
            Test::LessThanOrEqual(TagKey("#key 123?".into()), TagNumericValue::Float(2.7)),
        );
        parse_all_and_expect(
            Test::parse,
            "[_key > -15]",
            Test::GreaterThan(TagKey("_key".into()), TagNumericValue::Integer(-15)),
        );
        parse_all_and_expect(
            Test::parse,
            "[KEY2 >= 4e-9]",
            Test::GreaterThanOrEqual(TagKey("KEY2".into()), TagNumericValue::Float(4e-9)),
        );
        parse_all_and_expect(
            Test::parse,
            "[key =~ \"a+bc*\"]",
            Test::Matches(
                TagKey("key".into()),
                Regex(regex::Regex::new("^a+bc*$").unwrap()),
            ),
        );
        parse_all_and_expect(Test::parse, "[key]", Test::Set(TagKey("key".into())));
        parse_all_and_expect(Test::parse, "[\"key\"]", Test::Set(TagKey("key".into())));
        parse_all_and_expect(Test::parse, "[!key]", Test::NotSet(TagKey("key".into())));
        parse_all_and_expect_error(Test::parse, "something", MapCssErrorKind::TestBlockExpected);
        parse_all_and_expect_error(
            Test::parse,
            "[]",
            MapCssErrorKind::TagKeyOrExclamationExpected,
        );
        parse_all_and_expect_error(
            Test::parse,
            "[key > value]",
            MapCssErrorKind::TagNumericValueExpected,
        );
        parse_all_and_expect_error(Test::parse, "[key=]", MapCssErrorKind::TagValueExpected);
        parse_all_and_expect_basic_error(
            Test::parse,
            "[!key=value]",
            BasicParseErrorKind::UnexpectedToken(Token::Delim('=')),
        );
        parse_all_and_expect_error(Test::parse, "[!=7]", MapCssErrorKind::TagKeyExpected);
        parse_all_and_expect_error(Test::parse, "[a==3]", MapCssErrorKind::TagValueExpected);
        parse_all_and_expect_error(
            Test::parse,
            "[a~=3]",
            MapCssErrorKind::InvalidTokenInsideTest,
        );
        parse_all_and_expect_basic_error(
            Test::parse,
            "[key=value something]",
            BasicParseErrorKind::UnexpectedToken(Token::Ident("something".into())),
        );
    }

    #[test]
    fn basic_selector_parsing() {
        parse_all_and_expect(
            BasicSelector::parse,
            "node",
            BasicSelector::new(ObjectType::Node, vec![]),
        );
        parse_all_and_expect(
            BasicSelector::parse,
            "node [key=value]",
            BasicSelector::new(
                ObjectType::Node,
                vec![Test::Equal(
                    TagKey("key".into()),
                    TagValue::String(TagStringValue("value".into())),
                )],
            ),
        );
        parse_all_and_expect(
            BasicSelector::parse,
            "*[key=value]",
            BasicSelector::new(
                ObjectType::Any,
                vec![Test::Equal(
                    TagKey("key".into()),
                    TagValue::String(TagStringValue("value".into())),
                )],
            ),
        );
        parse_all_and_expect(
            BasicSelector::parse,
            "node[k1][k2][k3]",
            BasicSelector::new(
                ObjectType::Node,
                vec![
                    Test::Set(TagKey("k1".into())),
                    Test::Set(TagKey("k2".into())),
                    Test::Set(TagKey("k3".into())),
                ],
            ),
        );
        parse_all_and_expect_error(
            BasicSelector::parse,
            "[k1]",
            MapCssErrorKind::ObjectTypeExpected,
        );
        parse_all_and_expect_error(
            BasicSelector::parse,
            "node[something < invalid]",
            MapCssErrorKind::TagNumericValueExpected,
        );
        parse_all_and_expect_remaining(BasicSelector::parse, "way node", " node");
        parse_all_and_expect_remaining(
            BasicSelector::parse,
            "relation [tag] way [tag]",
            " way [tag]",
        );
        parse_all_and_expect_remaining(BasicSelector::parse, "area[k=v] {}", " {}");
    }

    #[test]
    fn selector_chain_parsing() {
        parse_all_and_expect(
            SelectorChain::parse,
            "node",
            SelectorChain::new(vec![BasicSelector::new(ObjectType::Node, vec![])]),
        );
        parse_all_and_expect(
            SelectorChain::parse,
            "way node [tag]",
            SelectorChain::new(vec![
                BasicSelector::new(ObjectType::Way, vec![]),
                BasicSelector::new(ObjectType::Node, vec![Test::Set(TagKey("tag".into()))]),
            ]),
        );
        parse_all_and_expect(
            SelectorChain::parse,
            "relation[k1]way[k2][k3]node[k4]",
            SelectorChain::new(vec![
                BasicSelector::new(ObjectType::Relation, vec![Test::Set(TagKey("k1".into()))]),
                BasicSelector::new(
                    ObjectType::Way,
                    vec![
                        Test::Set(TagKey("k2".into())),
                        Test::Set(TagKey("k3".into())),
                    ],
                ),
                BasicSelector::new(ObjectType::Node, vec![Test::Set(TagKey("k4".into()))]),
            ]),
        );
        parse_all_and_expect(
            SelectorChain::parse,
            "*[k1]*",
            SelectorChain::new(vec![
                BasicSelector::new(ObjectType::Any, vec![Test::Set(TagKey("k1".into()))]),
                BasicSelector::new(ObjectType::Any, vec![]),
            ]),
        );
        parse_all_and_expect_error(
            SelectorChain::parse,
            "node something",
            MapCssErrorKind::ObjectTypeExpected,
        );
    }

    #[test]
    fn selector_alternatives_parsing() {
        parse_all_and_expect(
            SelectorAlternatives::parse,
            "node",
            SelectorAlternatives::new(vec![SelectorChain::new(vec![BasicSelector::new(
                ObjectType::Node,
                vec![],
            )])]),
        );
        parse_all_and_expect(
            SelectorAlternatives::parse,
            "node, way",
            SelectorAlternatives::new(vec![
                SelectorChain::new(vec![BasicSelector::new(ObjectType::Node, vec![])]),
                SelectorChain::new(vec![BasicSelector::new(ObjectType::Way, vec![])]),
            ]),
        );
        parse_all_and_expect(
            SelectorAlternatives::parse,
            "way[k1] node, way[k2], node",
            SelectorAlternatives::new(vec![
                SelectorChain::new(vec![
                    BasicSelector::new(ObjectType::Way, vec![Test::Set(TagKey("k1".into()))]),
                    BasicSelector::new(ObjectType::Node, vec![]),
                ]),
                SelectorChain::new(vec![BasicSelector::new(
                    ObjectType::Way,
                    vec![Test::Set(TagKey("k2".into()))],
                )]),
                SelectorChain::new(vec![BasicSelector::new(ObjectType::Node, vec![])]),
            ]),
        );
        parse_all_and_expect_error(
            SelectorAlternatives::parse,
            "something",
            MapCssErrorKind::ObjectTypeExpected,
        );
    }
}
