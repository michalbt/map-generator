use slotmap::new_key_type;

use crate::{
    location::Location,
    object::{OsmId, Tags},
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
