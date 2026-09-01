use crate::types::selector::SelectorAlternatives;

#[derive(Clone, Debug, PartialEq)]
pub struct Stylesheet {
    rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rule {
    selector: SelectorAlternatives,
    declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Declaration {
    property_name: String,
    property_value: PropertyValue,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PropertyValue {
    String(String),
    Integer(i32),
    Float(f32),
    Percentage(f32),
    Color(u8, u8, u8, f32),
    Url(String),
    IntegerList(Vec<i32>),
}
