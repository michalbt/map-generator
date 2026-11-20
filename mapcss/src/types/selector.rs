use std::{collections::HashMap, str::FromStr};

use map_data::{AreaSource, KeyedMapping, ObjectKey, RelationRoleKey, Storage, TagKey};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SelectorAlternatives {
    alternatives: Vec<SelectorChain>,
}

impl SelectorAlternatives {
    pub fn new(alternatives: Vec<SelectorChain>) -> Self {
        Self { alternatives }
    }

    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        self.alternatives
            .iter()
            .any(|chain| chain.matches(key, storage))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SelectorChain {
    chain: Vec<BasicSelector>,
}

impl SelectorChain {
    pub fn new(chain: Vec<BasicSelector>) -> Self {
        Self { chain }
    }

    fn relation_role_matches(
        role_tests: &[RoleTest],
        parent_key: ObjectKey,
        child_key: ObjectKey,
        storage: &Storage,
    ) -> bool {
        if role_tests.is_empty() {
            true
        } else if let ObjectKey::Relation(relation_key) = parent_key {
            let relation_role = storage[relation_key]
                .members()
                .iter()
                .find(|m| m.object == child_key)
                .expect("child must be contained in parent")
                .role;
            role_tests.iter().all(|t| t.matches(relation_role))
        } else {
            // Not in a relation => all role tests fail
            false
        }
    }

    fn selector_subchain_matches(
        selector: &BasicSelector,
        parent_subchain: &[BasicSelector],
        child_role_tests: Option<(ObjectKey, &[RoleTest])>,
        key: ObjectKey,
        storage: &Storage,
    ) -> bool {
        selector.matches(key, storage)
            && child_role_tests.is_none_or(|(child_key, role_tests)| {
                Self::relation_role_matches(role_tests, key, child_key, storage)
            })
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
                        Some((key, &selector.role_tests)),
                        parent_key,
                        storage,
                    )
                })
            } else {
                true
            }
    }

    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        if let Some((last_selector, parent_subchain)) = self.chain.split_last() {
            Self::selector_subchain_matches(last_selector, parent_subchain, None, key, storage)
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BasicSelector {
    object_type: ObjectType,
    tests: Vec<TagTest>,
    role_tests: Vec<RoleTest>,
}

impl BasicSelector {
    pub fn new(object_type: ObjectType, tests: Vec<TagTest>, role_tests: Vec<RoleTest>) -> Self {
        Self {
            object_type,
            tests,
            role_tests,
        }
    }

    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        self.object_type.matches(key, storage) && {
            let object_tags = storage[key].tags();
            self.tests.iter().all(|test| test.matches(&object_tags))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectType {
    Node,
    Way,
    Relation,
    Area,
    Line,
    Canvas,
    Any,
}

impl ObjectType {
    // TODO: refactor after Area removal, ideally to not use &Storage
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
pub(crate) enum TagTest {
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

impl TagTest {
    fn matches(&self, object_tags: &HashMap<TagKey, String>) -> bool {
        match self {
            Self::Set(tag_key) => object_tags.contains_key(tag_key),
            Self::NotSet(tag_key) => !object_tags.contains_key(tag_key),
            Self::Equal(tag_key, tag_value) => object_tags
                .get(tag_key)
                .is_some_and(|val| *tag_value == **val),
            Self::NotEqual(tag_key, tag_value) => object_tags
                .get(tag_key)
                .is_some_and(|val| *tag_value != **val),
            Self::LessThan(tag_key, tag_value) => object_tags
                .get(tag_key)
                .and_then(|s| s.parse::<TagNumericValue>().ok())
                .is_some_and(|val| val < *tag_value),
            Self::LessThanOrEqual(tag_key, tag_value) => object_tags
                .get(tag_key)
                .and_then(|s| s.parse::<TagNumericValue>().ok())
                .is_some_and(|val| val <= *tag_value),
            Self::GreaterThan(tag_key, tag_value) => object_tags
                .get(tag_key)
                .and_then(|s| s.parse::<TagNumericValue>().ok())
                .is_some_and(|val| val > *tag_value),
            Self::GreaterThanOrEqual(tag_key, tag_value) => object_tags
                .get(tag_key)
                .and_then(|s| s.parse::<TagNumericValue>().ok())
                .is_some_and(|val| val >= *tag_value),
            Self::Matches(tag_key, Regex(regex)) => object_tags
                .get(tag_key)
                .is_some_and(|val| regex.is_match(val)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RoleTest {
    RoleSet,
    RoleNotSet,
    RoleEqual(RelationRoleKey),
    RoleNotEqual(RelationRoleKey),
}

impl RoleTest {
    fn matches(&self, role: Option<RelationRoleKey>) -> bool {
        const EMPTY_ROLE_KEY: RelationRoleKey = KeyedMapping::<RelationRoleKey>::EMPTY_ROLE_KEY;
        match self {
            Self::RoleSet => role != Some(EMPTY_ROLE_KEY),
            Self::RoleNotSet => role == Some(EMPTY_ROLE_KEY),
            Self::RoleEqual(key) => role == Some(*key),
            Self::RoleNotEqual(key) => role != Some(*key),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagValue {
    Numeric(TagNumericValue),
    String(TagStringValue),
}

impl PartialEq<str> for TagValue {
    fn eq(&self, other: &str) -> bool {
        match self {
            TagValue::Numeric(numeric_value) => other
                .parse::<TagNumericValue>()
                .is_ok_and(|val| val == *numeric_value),
            TagValue::String(TagStringValue(string_value)) => *other == *string_value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagNumericValue {
    Integer(i32),
    Float(f32),
}

impl FromStr for TagNumericValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(integer) = s.parse::<i32>() {
            Ok(Self::Integer(integer))
        } else if let Ok(float) = s.parse::<f32>() {
            Ok(Self::Float(float))
        } else {
            Err(())
        }
    }
}

impl PartialOrd<Self> for TagNumericValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Integer(left), Self::Integer(right)) => left.partial_cmp(right),
            (Self::Float(left), Self::Float(right)) => left.partial_cmp(right),
            (Self::Integer(left), Self::Float(right)) => {
                (*left as f64).partial_cmp(&(*right as f64))
            }
            (Self::Float(left), Self::Integer(right)) => {
                (*left as f64).partial_cmp(&(*right as f64))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagStringValue(pub String);

#[derive(Clone, Debug)]
pub(crate) struct Regex(pub regex::Regex);

impl PartialEq for Regex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}
