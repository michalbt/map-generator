use std::ops::{Index, IndexMut};

use slotmap::SlotMap;

use crate::{
    area::{Area, AreaKey, Ring},
    location::Location,
    node::{Node, NodeKey},
    object::{Object, ObjectKey},
    relation::{Relation, RelationKey, RelationMember},
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
        let way_key = self.ways.insert(way);
        for &node_key in self.ways[way_key].nodes() {
            self.nodes[node_key].add_containing_way(way_key);
        }
        way_key
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

    pub fn contains_area(&self, key: AreaKey) -> bool {
        self.areas.contains_key(key)
    }

    pub fn insert_area(&mut self, area: Area) -> AreaKey {
        let area_key = self.areas.insert(area);
        for ring in self.areas[area_key].rings() {
            for &way_key in &ring.ways {
                self.ways[way_key].add_formed_area(area_key);
            }
        }
        area_key
    }

    pub fn create_area(&mut self, rings: Vec<Ring>) -> AreaKey {
        self.insert_area(Area::new(None, rings))
    }

    pub fn add_ring_to_area(&mut self, area_key: AreaKey, ring: Ring) {
        for &way_key in &ring.ways {
            self[way_key].add_formed_area(area_key);
        }
        self[area_key].rings_mut().push(ring);
    }

    pub fn remove_ring_from_area(&mut self, area_key: AreaKey, ring_index: usize) -> Ring {
        let ring = self[area_key].rings_mut().remove(ring_index);
        for &way_key in &ring.ways {
            self[way_key].remove_formed_area(area_key);
        }
        ring
    }

    pub fn contains_relation(&self, key: RelationKey) -> bool {
        self.relations.contains_key(key)
    }

    pub fn insert_relation(&mut self, relation: Relation) -> RelationKey {
        let members_cloned = relation.members().to_vec();
        let relation_key = self.relations.insert(relation);
        for member in members_cloned {
            self[member.object].add_containing_relation(relation_key);
        }
        relation_key
    }

    pub fn create_relation(&mut self, members: Vec<RelationMember>) -> RelationKey {
        self.insert_relation(Relation::new(None, members))
    }

    pub fn append_member_to_relation(&mut self, relation_key: RelationKey, member: RelationMember) {
        let member_object_key = member.object;
        self[relation_key].members_mut().push(member);
        self[member_object_key].add_containing_relation(relation_key);
    }

    pub fn insert_member_to_relation(
        &mut self,
        relation_key: RelationKey,
        member: RelationMember,
        index: usize,
    ) {
        let member_object_key = member.object;
        self[relation_key].members_mut().insert(index, member);
        self[member_object_key].add_containing_relation(relation_key);
    }

    pub fn remove_member_from_relation(
        &mut self,
        relation_key: RelationKey,
        member_index: usize,
    ) -> RelationMember {
        let member = self[relation_key].members_mut().remove(member_index);
        self[member.object].remove_containing_relation(relation_key);
        member
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

impl Index<ObjectKey> for Storage {
    type Output = dyn Object;

    fn index(&self, index: ObjectKey) -> &Self::Output {
        match index {
            ObjectKey::Node(node_key) => &self[node_key],
            ObjectKey::Way(way_key) => &self[way_key],
            ObjectKey::Area(area_key) => &self[area_key],
            ObjectKey::Relation(relation_key) => &self[relation_key],
        }
    }
}

impl IndexMut<ObjectKey> for Storage {
    fn index_mut(&mut self, index: ObjectKey) -> &mut Self::Output {
        match index {
            ObjectKey::Node(node_key) => &mut self[node_key],
            ObjectKey::Way(way_key) => &mut self[way_key],
            ObjectKey::Area(area_key) => &mut self[area_key],
            ObjectKey::Relation(relation_key) => &mut self[relation_key],
        }
    }
}
