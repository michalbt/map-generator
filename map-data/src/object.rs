use std::{collections::HashMap, fmt::Debug};

use derive_more::From;

use crate::{area::AreaKey, node::NodeKey, relation::RelationKey, way::WayKey};

pub type OsmId = Option<i64>;

pub type Tags = HashMap<String, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, From)]
pub enum ObjectKey {
    Node(NodeKey),
    Way(WayKey),
    Area(AreaKey),
    Relation(RelationKey),
}

#[allow(private_bounds)]
pub trait Object: TrackContainingRelations {
    fn osm_id(&self) -> OsmId;

    fn tags(&self) -> &Tags;

    fn tags_mut(&mut self) -> &mut Tags;

    fn containing_relations(&self) -> &[RelationKey];

    fn has_tag(&self, tag: &str) -> bool {
        self.tags().contains_key(tag)
    }

    fn get_tag(&self, tag: &str) -> Option<&str> {
        self.tags().get(tag).map(|s| s.as_str())
    }

    fn set_tag(&mut self, tag: String, value: String) {
        self.tags_mut().insert(tag, value);
    }

    fn remove_tag(&mut self, tag: &str) {
        self.tags_mut().remove(tag);
    }
}

/// This trait is separate from Object because its methods need to be public only in this crate
pub(crate) trait TrackContainingRelations {
    fn add_containing_relation(&mut self, key: RelationKey);

    fn remove_containing_relation(&mut self, key: RelationKey);
}

macro_rules! impl_object {
    ($t:ty) => {
        impl $crate::object::Object for $t {
            fn osm_id(&self) -> $crate::object::OsmId {
                self.osm_id
            }

            fn tags(&self) -> &$crate::object::Tags {
                &self.tags
            }

            fn tags_mut(&mut self) -> &mut $crate::object::Tags {
                &mut self.tags
            }

            fn containing_relations(&self) -> &[RelationKey] {
                &self.containing_relations
            }
        }

        impl $crate::object::TrackContainingRelations for $t {
            fn add_containing_relation(&mut self, key: RelationKey) {
                self.containing_relations.push(key);
            }

            fn remove_containing_relation(&mut self, key: RelationKey) {
                let index = self
                    .containing_relations
                    .iter()
                    .position(|k| *k == key)
                    .expect(concat!(
                        "specified RelationKey is not a containing relation for this ",
                        stringify!($t)
                    ));
                self.containing_relations.swap_remove(index);
            }
        }
    };
}
pub(crate) use impl_object;
