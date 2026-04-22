use std::collections::HashSet;

use crate::{
    ids::{AreaId, RelationId, WayId},
    object::Tags,
};

#[derive(Clone, Debug)]
pub struct Area {
    id: AreaId,
    tags: Tags,
    rings: Vec<Ring>,
    containing_relations: HashSet<RelationId>,
}

#[derive(Clone, Debug)]
pub struct Ring {
    pub role: RingRole,
    pub ways: Vec<WayId>,
}

#[derive(Clone, Copy, Debug)]
pub enum RingRole {
    Outer,
    Inner,
}
