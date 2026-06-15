use std::collections::HashMap;

use crate::{
    area::Area,
    ids::{AreaId, IdProvider, NodeId, RelationId, WayId},
    node::Node,
    relation::Relation,
    span::Span,
    way::Way,
};

#[derive(Debug)]
pub struct Storage {
    nodes: HashMap<NodeId, Node>,
    ways: HashMap<WayId, Way>,
    areas: HashMap<AreaId, Area>,
    relations: HashMap<RelationId, Relation>,
    id_provider: IdProvider,
    map_span: Span,
}

impl Storage {
    pub fn new(map_span: Span) -> Storage {
        Storage {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            areas: HashMap::new(),
            relations: HashMap::new(),
            id_provider: IdProvider::new(),
            map_span,
        }
    }
}
