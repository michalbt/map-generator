use crate::types::{declaration::Declaration, selector::SelectorAlternatives};

#[derive(Clone, Debug, PartialEq)]
pub struct Stylesheet {
    pub(crate) rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rule {
    pub(crate) selector: SelectorAlternatives,
    pub(crate) declarations: Vec<Declaration>,
}
