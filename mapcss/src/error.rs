use cssparser::ParseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    InvalidHashColor,
    ColorUnitFloatExpected,
    ColorUnitFloatOutOfRange,
    CommaExpected,
    ColorAlphaUnitFloatExpected,
    ColorAlphaUnitFloatOutOfRange,
    ColorExpected,
    CommaSeparatedIntegerListExpected,
    CommaSeparatedIntegerListTooShort,
    IntegerExpected,
    PropertyValueExpected,
    PropertyNameExpected,
    ColonExpected,
    SemicolonExpected,
    DeclarationBlockExpected,
}

pub type MapCssParseError<'i> = ParseError<'i, MapCssErrorKind>;
