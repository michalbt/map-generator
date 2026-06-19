use std::collections::HashMap;

use slotmap::new_key_type;

use crate::{
    area::AreaKey,
    node::NodeKey,
    object::{OsmId, Tags, impl_object},
    relation::RelationKey,
};

new_key_type! { pub struct WayKey; }

#[derive(Clone, Debug)]
pub struct Way {
    osm_id: OsmId,
    tags: Tags,
    nodes: Vec<NodeKey>,
    formed_areas: Vec<AreaKey>,
    containing_relations: Vec<RelationKey>,
}

impl Way {
    pub fn new(osm_id: OsmId, nodes: Vec<NodeKey>) -> Self {
        Self {
            osm_id,
            tags: HashMap::new(),
            nodes,
            formed_areas: vec![],
            containing_relations: vec![],
        }
    }

    pub fn nodes(&self) -> &[NodeKey] {
        &self.nodes
    }

    pub(crate) fn nodes_mut(&mut self) -> &mut Vec<NodeKey> {
        &mut self.nodes
    }
}

impl_object!(Way);
