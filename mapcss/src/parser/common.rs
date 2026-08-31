use cssparser::Parser;

pub(crate) fn parse_identifier(input: &mut Parser) -> Result<String, ()> {
    input
        .expect_ident()
        .map(ToString::to_string)
        .map_err(|_| ())
}

pub(crate) fn parse_quoted_string(input: &mut Parser) -> Result<String, ()> {
    input
        .expect_string()
        .map(ToString::to_string)
        .map_err(|_| ())
}

pub(crate) fn parse_identifier_or_quoted_string(input: &mut Parser) -> Result<String, ()> {
    input
        .expect_ident_or_string()
        .map(ToString::to_string)
        .map_err(|_| ())
}
