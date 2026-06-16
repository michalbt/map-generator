use slotmap::new_key_type;

use crate::object::{ObjectHandle, OsmId, Tags, impl_object};

new_key_type! { pub struct RelationKey; }

#[derive(Clone, Debug)]
pub struct Relation {
    osm_id: OsmId,
    tags: Tags,
    members: Vec<RelationMember>,
    containing_relations: Vec<RelationKey>,
}

impl_object!(Relation);

#[derive(Clone, Debug)]
pub struct RelationMember {
    pub object: ObjectHandle,
    pub role: Option<String>,
}
