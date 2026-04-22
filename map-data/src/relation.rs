use std::collections::HashSet;

use crate::{
    ids::RelationId,
    object::{ObjectHandle, Tags},
};

#[derive(Clone, Debug)]
pub struct Relation {
    id: RelationId,
    tags: Tags,
    members: Vec<RelationMember>,
    containing_relations: HashSet<RelationId>,
}

#[derive(Clone, Debug)]
pub struct RelationMember {
    pub object: ObjectHandle,
    pub role: Option<String>,
}
