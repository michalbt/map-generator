use slotmap::new_key_type;

use crate::{
    area::AreaKey,
    node::NodeKey,
    object::{OsmId, Tags},
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
