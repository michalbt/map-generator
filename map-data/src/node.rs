use std::collections::HashSet;

use crate::{
    ids::{NodeId, RelationId, WayId},
    location::Location,
    object::Tags,
};

#[derive(Clone, Debug)]
pub struct Node {
    id: NodeId,
    location: Location,
    tags: Tags,
    containing_ways: HashSet<WayId>,
    containing_relations: HashSet<RelationId>,
}
