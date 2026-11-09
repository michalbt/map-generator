#![allow(dead_code)]

mod area;
mod keyed_mapping;
mod location;
mod node;
mod object;
mod relation;
mod span;
mod storage;
mod tags;
mod way;

pub(crate) use keyed_mapping::use_keyed_mapping_with_well_known_keys;

pub use crate::area::AreaSource;
pub use crate::keyed_mapping::KeyedMapping;
pub use crate::object::ObjectKey;
pub use crate::storage::Storage;
pub use crate::tags::TagKey;
