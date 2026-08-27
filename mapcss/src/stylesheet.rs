use crate::selector::SelectorAlternatives;

pub struct Stylesheet {
    rules: Vec<Rule>,
}

struct Rule {
    selector: SelectorAlternatives,
    declarations: Vec<Declaration>,
}

struct Declaration {
    property_name: String,
    property_value: String,
}
