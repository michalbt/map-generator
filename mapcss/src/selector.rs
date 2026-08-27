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
    LessThan(TagKey, TagValue),
    LessThanOrEqual(TagKey, TagValue),
    GreaterThan(TagKey, TagValue),
    GreaterThanOrEqual(TagKey, TagValue),
    Matches(TagKey, Regex),
}

impl Selector for Test {
    fn matches(&self, key: ObjectKey, storage: &Storage) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TagKey(pub String);

#[derive(Clone, Debug, PartialEq)]
struct TagValue(pub String);

#[derive(Clone, Debug, PartialEq)]
struct Regex(pub String);
