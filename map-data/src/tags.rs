use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type};

pub type Tags = HashMap<TagKey, String>;

new_key_type! { pub struct TagKey; }

#[derive(Debug)]
pub struct TagMapping {
    forward: SlotMap<TagKey, String>,
    reverse: HashMap<String, TagKey>,
    well_known_tag_storage: WellKnownTagStorage,
}

macro_rules! use_well_known_tags {
    ($(($tag_str:literal, $tag_ident:ident, $tag_enum_variant:ident)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum WellKnownTag {
            $($tag_enum_variant,)*
        }

        #[derive(Debug)]
        struct WellKnownTagStorage {
            $($tag_ident: TagKey,)*
        }

        impl TagMapping {
            pub fn new() -> Self {
                let mut forward = SlotMap::with_key();
                let mut reverse = HashMap::new();
                $({
                    let key = forward.insert($tag_str.to_string());
                    reverse.insert($tag_str.to_string(), key);
                })*
                let well_known_tag_storage = WellKnownTagStorage {
                    $($tag_ident: reverse[$tag_str],)*
                };
                Self { forward, reverse, well_known_tag_storage }
            }

            pub fn get_well_known_tag_key(&self, tag: WellKnownTag) -> TagKey {
                match tag {
                    $(WellKnownTag::$tag_enum_variant => self.well_known_tag_storage.$tag_ident,)*
                }
            }
        }
    };
}

use_well_known_tags!(
    ("type", type_, Type),
    ("area", area, Area),
    ("highway", highway, Highway),
    ("barrier", barrier, Barrier),
);

impl TagMapping {
    pub fn get_or_insert(&mut self, tag: &str) -> TagKey {
        match self.reverse.get(tag) {
            Some(&tag_key) => tag_key,
            None => {
                let tag_key = self.forward.insert(tag.to_string());
                self.reverse.insert(tag.to_string(), tag_key);
                tag_key
            }
        }
    }

    pub fn get_key(&self, tag: &str) -> Option<TagKey> {
        self.reverse.get(tag).copied()
    }

    pub fn get_string(&self, key: TagKey) -> &str {
        &self.forward[key]
    }
}
