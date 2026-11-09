use std::collections::HashMap;
use std::fmt::Debug;

pub trait KeyedMappingKey: Debug + Copy {
    fn from_raw(raw: usize) -> Self;
    fn into_raw(self) -> usize;
}

#[derive(Debug)]
pub struct KeyedMapping<K: KeyedMappingKey> {
    pub(crate) forward: Vec<String>,
    pub(crate) reverse: HashMap<String, K>,
}

// Remove the $order argument and use {index()} after RFC 3086 is stabilized
// https://github.com/rust-lang/rust/issues/83527
macro_rules! use_keyed_mapping_with_well_known_keys {
    ($key_type:ty, $(($order:literal, $key_str:literal, $const_ident:ident)),* $(,)?) => {
        use $crate::keyed_mapping::KeyedMapping;

        impl KeyedMapping<$key_type> {
            $(
                pub const $const_ident: usize = $order;
            )*

            pub fn new() -> Self {
                let mut forward = Vec::new();
                let mut reverse = HashMap::new();
                $({
                    forward.push($key_str.to_string());
                    assert_eq!($order, forward.len());
                    reverse.insert($key_str.to_string(), <$key_type>::from_raw($order));
                })*
                Self { forward, reverse }
            }
        }

        impl Default for KeyedMapping<$key_type> {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

pub(crate) use use_keyed_mapping_with_well_known_keys;

impl<K: KeyedMappingKey> KeyedMapping<K> {
    pub fn get_or_insert(&mut self, tag: &str) -> K {
        match self.reverse.get(tag) {
            Some(&tag_key) => tag_key,
            None => {
                self.forward.push(tag.to_string());
                let tag_key = K::from_raw(self.forward.len());
                self.reverse.insert(tag.to_string(), tag_key);
                tag_key
            }
        }
    }

    pub fn get_key(&self, tag: &str) -> Option<K> {
        self.reverse.get(tag).copied()
    }

    pub fn get_string(&self, key: K) -> &str {
        &self.forward[key.into_raw()]
    }
}
