use std::{assert_matches, fmt::Debug};

use cssparser::{BasicParseErrorKind, ParseErrorKind, Parser, ParserInput};

use crate::error::{MapCssErrorKind, MapCssParseError};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParseAllError<'i> {
    ParseError(MapCssParseError<'i>),
    RemainingInput(&'i str),
}

pub(crate) fn parse_all<'i, T, F>(
    parser_function: F,
    input: &'i str,
) -> Result<T, ParseAllError<'i>>
where
    F: FnOnce(&mut Parser<'i, '_>) -> Result<T, MapCssParseError<'i>>,
{
    let mut parser_input = ParserInput::new(input);
    let mut parser = Parser::new(&mut parser_input);
    let result = parser_function(&mut parser).map_err(ParseAllError::ParseError)?;
    match parser.try_parse(|p| p.expect_exhausted()) {
        Ok(_) => Ok(result),
        Err(_) => Err(ParseAllError::RemainingInput(
            &input[parser.position().byte_index()..],
        )),
    }
}

pub(crate) fn parse_all_and_expect<'i, T, F>(parser_function: F, input: &'i str, expected_result: T)
where
    F: FnOnce(&mut Parser<'i, '_>) -> Result<T, MapCssParseError<'i>>,
    T: Debug + PartialEq,
{
    assert_eq!(parse_all(parser_function, input), Ok(expected_result));
}

pub(crate) fn parse_all_and_expect_error<'i, T, F>(
    parser_function: F,
    input: &'i str,
    expected_error_kind: MapCssErrorKind,
) where
    F: FnOnce(&mut Parser<'i, '_>) -> Result<T, MapCssParseError<'i>>,
    T: Debug + PartialEq,
{
    assert_matches!(
        parse_all(parser_function, input),
        Err(ParseAllError::ParseError(e))
            if e.kind == ParseErrorKind::Custom(expected_error_kind),
    );
}

pub(crate) fn parse_all_and_expect_basic_error<'i, T, F>(
    parser_function: F,
    input: &'i str,
    expected_error_kind: BasicParseErrorKind<'i>,
) where
    F: FnOnce(&mut Parser<'i, '_>) -> Result<T, MapCssParseError<'i>>,
    T: Debug + PartialEq,
{
    assert_matches!(
        parse_all(parser_function, input),
        Err(ParseAllError::ParseError(e))
            if e.kind == ParseErrorKind::Basic(expected_error_kind),
    );
}

pub(crate) fn parse_all_and_expect_remaining<'i, T, F>(
    parser_function: F,
    input: &'i str,
    expected_remaining_input: &'i str,
) where
    F: FnOnce(&mut Parser<'i, '_>) -> Result<T, MapCssParseError<'i>>,
    T: Debug + PartialEq,
{
    assert_eq!(
        parse_all(parser_function, input),
        Err(ParseAllError::RemainingInput(expected_remaining_input))
    );
}
