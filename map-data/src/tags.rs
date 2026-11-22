use std::collections::HashMap;

use crate::use_keyed_mapping_with_well_known_keys;

use_keyed_mapping_with_well_known_keys!(
    pub TagKey,
    (0, "type", TYPE_KEY),
    (1, "area", AREA_KEY),
    (2, "highway", HIGHWAY_KEY),
    (3, "barrier", BARRIER_KEY),
);

pub type Tags = HashMap<TagKey, String>;
