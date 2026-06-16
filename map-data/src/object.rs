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
