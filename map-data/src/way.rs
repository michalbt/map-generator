use std::{collections::HashMap, ops::Deref};

use slotmap::new_key_type;

use crate::{
    KeyedMapping, TagKey,
    node::NodeKey,
    object::{OsmId, impl_object},
    relation::RelationKey,
    tags::Tags,
};

const AREA_POSITIVE_VALUE: &str = "yes";
const AREA_NEGATIVE_VALUE: &str = "no";

new_key_type! { pub struct WayKey; }

#[derive(Clone, Debug, PartialEq)]
pub struct Way {
    osm_id: OsmId,
    tags: Tags,
    nodes: Vec<NodeKey>,
    containing_relations: Vec<RelationKey>,
}

impl Way {
    pub fn new(osm_id: OsmId, nodes: Vec<NodeKey>) -> Self {
        Self {
            osm_id,
            tags: HashMap::new(),
            nodes,
            containing_relations: vec![],
        }
    }

    pub fn nodes(&self) -> &[NodeKey] {
        &self.nodes
    }

    pub(crate) fn nodes_mut(&mut self) -> &mut Vec<NodeKey> {
        &mut self.nodes
    }

    pub fn defines_area(&self) -> bool {
        match (self.nodes.first(), self.nodes.last()) {
            (Some(first), Some(last)) => {
                first == last
                    && match self
                        .tags
                        .get(&KeyedMapping::<TagKey>::AREA_KEY)
                        .map(Deref::deref)
                    {
                        Some("yes") => true,
                        Some("no") => false,
                        _ => {
                            !self.tags.contains_key(&KeyedMapping::<TagKey>::HIGHWAY_KEY)
                                && !self.tags.contains_key(&KeyedMapping::<TagKey>::BARRIER_KEY)
                        }
                    }
            }
            _ => false,
        }
    }
}

impl_object!(Way);
