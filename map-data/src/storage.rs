use slotmap::SlotMap;

use crate::{
    area::{Area, AreaKey},
    location::Location,
    node::{Node, NodeKey},
    relation::{Relation, RelationKey},
    span::Span,
    way::{Way, WayKey},
};

#[derive(Debug)]
pub struct Storage {
    nodes: SlotMap<NodeKey, Node>,
    ways: SlotMap<WayKey, Way>,
    areas: SlotMap<AreaKey, Area>,
    relations: SlotMap<RelationKey, Relation>,
    map_span: Span,
}

impl Storage {
    pub fn new(map_span: Span) -> Storage {
        Storage {
            nodes: SlotMap::with_key(),
            ways: SlotMap::with_key(),
            areas: SlotMap::with_key(),
            relations: SlotMap::with_key(),
            map_span,
        }
    }

    pub fn contains_node(&self, key: NodeKey) -> bool {
        self.nodes.contains_key(key)
    }

    pub fn insert_node(&mut self, node: Node) -> NodeKey {
        self.nodes.insert(node)
    }

    pub fn create_node(&mut self, location: Location) -> NodeKey {
        self.insert_node(Node::new(None, location))
    }

    pub fn contains_way(&self, key: WayKey) -> bool {
        self.ways.contains_key(key)
    }

    pub fn insert_way(&mut self, way: Way) -> WayKey {
        self.ways.insert(way)
    }

    pub fn create_way(&mut self, nodes: Vec<NodeKey>) -> WayKey {
        self.insert_way(Way::new(None, nodes))
    }

    pub fn append_node_to_way(&mut self, way_key: WayKey, node_key: NodeKey) {
        self[way_key].nodes_mut().push(node_key);
        self[node_key].add_containing_way(way_key);
    }

    pub fn insert_node_to_way(&mut self, way_key: WayKey, node_key: NodeKey, index: usize) {
        self[way_key].nodes_mut().insert(index, node_key);
        self[node_key].add_containing_way(way_key);
    }

    pub fn remove_node_from_way(&mut self, way_key: WayKey, node_index: usize) {
        let node_key = self[way_key].nodes_mut().remove(node_index);
        self[node_key].remove_containing_way(way_key);
    }
}

macro_rules! impl_index {
    ($key:ty, $output:ty, $field:ident) => {
        impl std::ops::Index<$key> for Storage {
            type Output = $output;

            fn index(&self, index: $key) -> &Self::Output {
                &self.$field[index]
            }
        }

        impl std::ops::IndexMut<$key> for Storage {
            fn index_mut(&mut self, index: $key) -> &mut Self::Output {
                &mut self.$field[index]
            }
        }
    };
}

impl_index!(NodeKey, Node, nodes);
impl_index!(WayKey, Way, ways);
impl_index!(AreaKey, Area, areas);
impl_index!(RelationKey, Relation, relations);
