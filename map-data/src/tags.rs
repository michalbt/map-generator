use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TagKey(usize);

pub type Tags = HashMap<TagKey, String>;

#[derive(Debug)]
pub struct TagMapping {
    forward: Vec<String>,
    reverse: HashMap<String, TagKey>,
}

macro_rules! use_well_known_tags {
    ($(($order:literal, $tag_key_str:literal, $const_ident:ident)),* $(,)?) => {
        impl TagMapping {
            $(
                pub const $const_ident: usize = $order;
            )*

            pub fn new() -> Self {
                let mut forward = Vec::new();
                let mut reverse = HashMap::new();
                $({
                    forward.push($tag_key_str.to_string());
                    assert_eq!($order, forward.len());
                    reverse.insert($tag_key_str.to_string(), TagKey($order));
                })*
                Self { forward, reverse }
            }
        }
    };
}

// Remove the $order argument and use {index()} after RFC 3086 is stabilized
// https://github.com/rust-lang/rust/issues/83527
use_well_known_tags!(
    (0, "type", TYPE_KEY),
    (1, "area", AREA_KEY),
    (2, "highway", HIGHWAY_KEY),
    (3, "barrier", BARRIER_KEY),
);

impl TagMapping {
    pub fn get_or_insert(&mut self, tag: &str) -> TagKey {
        match self.reverse.get(tag) {
            Some(&tag_key) => tag_key,
            None => {
                self.forward.push(tag.to_string());
                let tag_key = TagKey(self.forward.len());
                self.reverse.insert(tag.to_string(), tag_key);
                tag_key
            }
        }
    }

    pub fn get_key(&self, tag: &str) -> Option<TagKey> {
        self.reverse.get(tag).copied()
    }

    pub fn get_string(&self, key: TagKey) -> &str {
        &self.forward[key.0]
    }
}

impl Default for TagMapping {
    fn default() -> Self {
        Self::new()
    }
}
