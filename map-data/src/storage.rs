use slotmap::SlotMap;

use crate::{
    area::{Area, AreaKey},
    node::{Node, NodeKey},
    relation::{Relation, RelationKey},
    span::Span,
    way::{Way, WayKey},
};

#[derive(Debug)]
pub struct Storage {
    nodes: SlotMap<NodeKey, Node>,
    ways: SlotMap<WayKey, Way>,
    areas: SlotMap<AreaKey, Area>,
    relations: SlotMap<RelationKey, Relation>,
    map_span: Span,
}

impl Storage {
    pub fn new(map_span: Span) -> Storage {
        Storage {
            nodes: SlotMap::with_key(),
            ways: SlotMap::with_key(),
            areas: SlotMap::with_key(),
            relations: SlotMap::with_key(),
            map_span,
        }
    }
}
