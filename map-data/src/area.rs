use slotmap::new_key_type;

use crate::{
    object::{OsmId, Tags},
    relation::RelationKey,
    way::WayKey,
};

new_key_type! { pub struct AreaKey; }

#[derive(Clone, Debug)]
pub struct Area {
    osm_id: OsmId,
    tags: Tags,
    rings: Vec<Ring>,
    containing_relations: Vec<RelationKey>,
}

#[derive(Clone, Debug)]
pub struct Ring {
    pub role: RingRole,
    pub ways: Vec<WayKey>,
}

#[derive(Clone, Copy, Debug)]
pub enum RingRole {
    Outer,
    Inner,
}
