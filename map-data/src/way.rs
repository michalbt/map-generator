use std::collections::HashSet;

use crate::{
    ids::{AreaId, NodeId, RelationId, WayId},
    object::Tags,
};

#[derive(Clone, Debug)]
pub struct Way {
    id: WayId,
    tags: Tags,
    nodes: Vec<NodeId>,
    formed_areas: HashSet<AreaId>,
    containing_relations: HashSet<RelationId>,
}
