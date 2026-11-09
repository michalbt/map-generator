use std::collections::HashMap;

use crate::{keyed_mapping::KeyedMappingKey, use_keyed_mapping_with_well_known_keys};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TagKey(usize);

impl KeyedMappingKey for TagKey {
    fn from_raw(raw: usize) -> Self {
        TagKey(raw)
    }

    fn into_raw(self) -> usize {
        self.0
    }
}

use_keyed_mapping_with_well_known_keys!(
    TagKey,
    (0, "type", TYPE_KEY),
    (1, "area", AREA_KEY),
    (2, "highway", HIGHWAY_KEY),
    (3, "barrier", BARRIER_KEY),
);

pub type Tags = HashMap<TagKey, String>;
