use std::collections::HashMap;

use crate::{area::AreaKey, node::NodeKey, relation::RelationKey, way::WayKey};

pub type OsmId = Option<i64>;

pub type Tags = HashMap<String, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ObjectHandle {
    Node(NodeKey),
    Way(WayKey),
    Area(AreaKey),
    Relation(RelationKey),
}

pub trait Object {
    fn osm_id(&self) -> OsmId;

    fn tags(&self) -> &Tags;

    fn tags_mut(&mut self) -> &mut Tags;

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
        }
    };
}
pub(crate) use impl_object;
