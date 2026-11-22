use std::{collections::HashMap, ops::Deref};

use slotmap::new_key_type;

use crate::{
    TagKey,
    object::{ObjectKey, OsmId, impl_object},
    tags::Tags,
    use_keyed_mapping_with_well_known_keys,
};

const RELATION_TYPE_MULTIPOLYGON: &str = "multipolygon";

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

    pub fn defines_area(&self) -> bool {
        self.tags.get(&TagKey::TYPE_KEY).map(Deref::deref) == Some(RELATION_TYPE_MULTIPOLYGON)
    }
}

impl_object!(Relation);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationMember {
    pub object: ObjectKey,
    /// `None` = has a role but the app does not care about it,
    /// `Some(EMPTY_ROLE_KEY)` = has an empty role
    pub role: Option<RelationRoleKey>,
}

impl RelationMember {
    pub fn new<K: Into<ObjectKey>>(key: K, role: Option<RelationRoleKey>) -> Self {
        Self {
            object: key.into(),
            role,
        }
    }
}

use_keyed_mapping_with_well_known_keys!(
    pub RelationRoleKey,
    (0, "", EMPTY_ROLE_KEY),
    (1, "outer", OUTER_KEY),
    (2, "inner", INNER_KEY),
);
