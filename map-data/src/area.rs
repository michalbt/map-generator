use std::collections::HashMap;

use slotmap::new_key_type;

use crate::{
    object::{OsmId, Tags, impl_object},
    relation::RelationKey,
    way::WayKey,
};

new_key_type! { pub struct AreaKey; }

#[derive(Clone, Debug, PartialEq)]
pub struct Area {
    osm_id: OsmId,
    source: AreaSource,
    tags: Tags,
    rings: Vec<Ring>,
    containing_relations: Vec<RelationKey>,
}

impl Area {
    pub fn new(osm_id: OsmId, source: AreaSource, rings: Vec<Ring>) -> Self {
        Self {
            osm_id,
            source,
            tags: HashMap::new(),
            rings,
            containing_relations: vec![],
        }
    }

    pub fn rings(&self) -> &[Ring] {
        &self.rings
    }

    pub fn rings_mut(&mut self) -> &mut Vec<Ring> {
        &mut self.rings
    }

    pub fn source(&self) -> AreaSource {
        self.source
    }
}

impl_object!(Area);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ring {
    pub role: RingRole,
    pub ways: Vec<WayKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingRole {
    Outer,
    Inner,
}

impl Ring {
    pub fn new_outer(ways: Vec<WayKey>) -> Self {
        Self {
            role: RingRole::Outer,
            ways,
        }
    }

    pub fn new_inner(ways: Vec<WayKey>) -> Self {
        Self {
            role: RingRole::Inner,
            ways,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaSource {
    Way,
    Relation,
    None,
}
