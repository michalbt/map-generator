use std::collections::HashMap;

use slotmap::new_key_type;

use crate::object::{ObjectKey, OsmId, Tags, impl_object};

new_key_type! { pub struct RelationKey; }

#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    osm_id: OsmId,
    tags: Tags,
    members: Vec<RelationMember>,
    containing_relations: Vec<RelationKey>,
}

impl Relation {
    pub fn new(osm_id: OsmId, members: Vec<RelationMember>) -> Self {
        Self {
            osm_id,
            tags: HashMap::new(),
            members,
            containing_relations: vec![],
        }
    }

    pub fn members(&self) -> &[RelationMember] {
        &self.members
    }

    pub fn members_mut(&mut self) -> &mut Vec<RelationMember> {
        &mut self.members
    }
}

impl_object!(Relation);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationMember {
    pub object: ObjectKey,
    pub role: Option<String>,
}

impl RelationMember {
    pub fn new<K: Into<ObjectKey>>(key: K, role: Option<String>) -> Self {
        Self {
            object: key.into(),
            role,
        }
    }
}
