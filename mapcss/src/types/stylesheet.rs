use crate::types::selector::SelectorAlternatives;

#[derive(Clone, Debug, PartialEq)]
pub struct Stylesheet {
    pub(crate) rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rule {
    pub(crate) selector: SelectorAlternatives,
    pub(crate) declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Declaration {
    pub(crate) property_name: String,
    pub(crate) property_value: PropertyValue,
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
