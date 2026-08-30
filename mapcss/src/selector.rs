use cssparser::Parser;
use map_data::{AreaSource, ObjectKey, Storage};

pub(crate) trait Selector {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SelectorAlternatives {
    alternatives: Vec<SelectorChain>,
}

impl Selector for SelectorAlternatives {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        self.alternatives
            .iter()
            .any(|chain| chain.matches(key, storage))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SelectorChain {
    chain: Vec<BasicSelector>,
}

impl Selector for SelectorChain {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        if let Some((last_selector, parent_subchain)) = self.chain.split_last() {
            Self::selector_subchain_matches(last_selector, parent_subchain, key, storage)
        } else {
            false
        }
    }
}

impl SelectorChain {
    fn selector_subchain_matches(
        selector: &BasicSelector,
        parent_subchain: &[BasicSelector],
        key: ObjectKey,
        storage: &Storage,
    ) -> bool {
        selector.matches(key, storage)
            && if let Some((parent_selector, parent_subchain_rest)) = parent_subchain.split_last() {
                let mut parent_objects = storage[key]
                    .containing_relations()
                    .iter()
                    .map(|&k| ObjectKey::Relation(k))
                    .collect::<Vec<_>>();
                if let ObjectKey::Node(node_key) = key {
                    parent_objects.extend(
                        storage[node_key]
                            .containing_ways()
                            .iter()
                            .map(|&k| ObjectKey::Way(k)),
                    );
                }
                if let ObjectKey::Way(way_key) = key {
                    parent_objects.extend(
                        storage[way_key]
                            .formed_areas()
                            .iter()
                            .map(|&k| ObjectKey::Area(k)),
                    )
                }
                parent_objects.iter().any(|&parent_key| {
                    Self::selector_subchain_matches(
                        parent_selector,
                        parent_subchain_rest,
                        parent_key,
                        storage,
                    )
                })
            } else {
                true
            }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct BasicSelector {
    object_type: ObjectType,
    tests: Vec<Test>,
}

impl Selector for BasicSelector {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        self.object_type.matches(key, storage)
            && self.tests.iter().all(|test| test.matches(key, storage))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObjectType {
    Node,
    Way,
    Relation,
    Area,
    Line,
    Canvas,
    Any,
}

impl ObjectType {
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        fn parse_identifier_object_type(input: &mut Parser) -> Result<ObjectType, ()> {
            if let Ok(identifier) = input.expect_ident() {
                match &**identifier {
                    "node" => Ok(ObjectType::Node),
                    "way" => Ok(ObjectType::Way),
                    "relation" => Ok(ObjectType::Relation),
                    "area" => Ok(ObjectType::Area),
                    "line" => Ok(ObjectType::Line),
                    "canvas" => Ok(ObjectType::Canvas),
                    _ => Err(()),
                }
            } else {
                Err(())
            }
        }

        if let Ok(object_type) = input.try_parse(parse_identifier_object_type) {
            Ok(object_type)
        } else if let Ok(()) = input.expect_delim('*') {
            Ok(ObjectType::Any)
        } else {
            Err(())
        }
    }
}

impl Selector for ObjectType {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        match (*self, key) {
            (Self::Node, ObjectKey::Node(_)) => true,
            (Self::Node, _) => false,
            (Self::Way, ObjectKey::Way(_)) => true,
            (Self::Way, ObjectKey::Area(area_key)) => storage[area_key].source() == AreaSource::Way,
            (Self::Way, _) => false,
            (Self::Relation, ObjectKey::Relation(_)) => true,
            (Self::Relation, ObjectKey::Area(area_key)) => {
                storage[area_key].source() == AreaSource::Relation
            }
            (Self::Relation, _) => false,
            (Self::Area, ObjectKey::Area(_)) => true,
            (Self::Area, _) => false,
            (Self::Line, ObjectKey::Way(_)) => true,
            (Self::Line, _) => false,
            (Self::Canvas, _) => false,
            (Self::Any, _) => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Test {
    Set(TagKey),
    NotSet(TagKey),
    Equal(TagKey, TagValue),
    NotEqual(TagKey, TagValue),
    LessThan(TagKey, TagNumericValue),
    LessThanOrEqual(TagKey, TagNumericValue),
    GreaterThan(TagKey, TagNumericValue),
    GreaterThanOrEqual(TagKey, TagNumericValue),
    Matches(TagKey, Regex),
}

impl Test {
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        todo!()
    }
}

impl Selector for Test {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TagKey(pub String);

impl TagKey {
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        input
            .expect_ident_or_string()
            .map(|s| TagKey(s.to_string()))
            .map_err(|_| ())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TagValue {
    Numeric(TagNumericValue),
    String(TagStringValue),
}

impl TagValue {
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        if let Ok(numeric) = input.try_parse(TagNumericValue::parse) {
            Ok(Self::Numeric(numeric))
        } else if let Ok(string) = TagStringValue::parse(input) {
            Ok(Self::String(string))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TagNumericValue {
    Integer(i32),
    Float(f32),
}

impl TagNumericValue {
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        if let Ok(integer) = input.try_parse(|inp| inp.expect_integer()) {
            Ok(Self::Integer(integer))
        } else if let Ok(float) = input.expect_number() {
            Ok(Self::Float(float))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TagStringValue(pub String);

impl TagStringValue {
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        input
            .expect_ident_or_string()
            .map(|s| TagStringValue(s.to_string()))
            .map_err(|_| ())
    }
}

#[derive(Clone, Debug)]
struct Regex(pub regex::Regex);

impl Regex {
    /// Only quoted regexes are supported
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        input
            .expect_string()
            .map_err(|_| ())
            .and_then(|s| regex::Regex::new(s).map_err(|_| ()))
            .map(Regex)
    }
}

impl PartialEq for Regex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}
