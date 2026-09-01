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
    InvalidHashColor,
    ColorUnitFloatExpected,
    ColorUnitFloatOutOfRange,
    CommaExpected,
    ColorAlphaUnitFloatExpected,
    ColorAlphaUnitFloatOutOfRange,
    ColorExpected,
}

pub type MapCssParseError<'i> = ParseError<'i, MapCssErrorKind>;
