use std::collections::HashMap;

use slotmap::new_key_type;

use crate::{
    area::AreaKey,
    node::NodeKey,
    object::{OsmId, Tags, impl_object},
    relation::RelationKey,
};

new_key_type! { pub struct WayKey; }

#[derive(Clone, Debug, PartialEq)]
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

    pub fn formed_areas(&self) -> &[AreaKey] {
        &self.formed_areas
    }

    pub(crate) fn add_formed_area(&mut self, key: AreaKey) {
        self.formed_areas.push(key);
    }

    pub(crate) fn remove_formed_area(&mut self, key: AreaKey) {
        let index = self
            .formed_areas
            .iter()
            .position(|k| *k == key)
            .expect("specified AreaKey is not a formed area for this Way");
        self.formed_areas.swap_remove(index);
    }
}

impl_object!(Way);
