use cssparser::ParseError;

pub enum MapCssErrorKind {
    ObjectTypeExpected,
    TagKeyExpected,
    TagValueExpected,
    TagNumericValueExpected,
    TagStringValueExpected,
    RegexExpected,
    InvalidRegex,
    TestBlockExpected,
    TagKeyOrExclamationExpected,
    InvalidTokenInsideTest,
    SelectorChainExpected,
}

pub type MapCssParseError<'i> = ParseError<'i, MapCssErrorKind>;
