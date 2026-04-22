use std::collections::HashMap;

use crate::ids::{AreaId, NodeId, RelationId, WayId};

pub type Tags = HashMap<String, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ObjectHandle {
    Node(NodeId),
    Way(WayId),
    Area(AreaId),
    Relation(RelationId),
}
