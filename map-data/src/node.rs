use std::collections::HashMap;

use slotmap::new_key_type;

use crate::{
    location::Location,
    object::{OsmId, Tags, impl_object},
    relation::RelationKey,
    way::WayKey,
};

new_key_type! { pub struct NodeKey; }

#[derive(Clone, Debug)]
pub struct Node {
    osm_id: OsmId,
    location: Location,
    tags: Tags,
    containing_ways: Vec<WayKey>,
    containing_relations: Vec<RelationKey>,
}

impl Node {
    pub fn new(osm_id: OsmId, location: Location) -> Self {
        Self {
            osm_id,
            location,
            tags: HashMap::new(),
            containing_ways: vec![],
            containing_relations: vec![],
        }
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn set_location(&mut self, new_location: Location) {
        self.location = new_location;
    }

    pub(crate) fn add_containing_way(&mut self, key: WayKey) {
        self.containing_ways.push(key);
    }

    pub(crate) fn remove_containing_way(&mut self, key: WayKey) {
        let index = self
            .containing_ways
            .iter()
            .position(|k| *k == key)
            .expect("node does not contain specified way");
        self.containing_ways.swap_remove(index);
    }
}

impl_object!(Node);
