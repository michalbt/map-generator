use std::ops::{Index, IndexMut};

use slotmap::SlotMap;

use crate::{
    area::{Area, AreaKey, AreaSource, Ring},
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

    pub fn delete_node(&mut self, key: NodeKey) -> Result<Node, ObjectDeleteError> {
        let node = self.nodes.get(key).ok_or(ObjectDeleteError::NotPresent)?;
        if !node.containing_ways().is_empty() {
            Err(ObjectDeleteError::ContainedInWay)
        } else if !node.containing_relations().is_empty() {
            Err(ObjectDeleteError::ContainedInRelation)
        } else {
            Ok(self
                .nodes
                .remove(key)
                .expect("already checked that the object is present"))
        }
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeKey, &Node)> {
        self.nodes.iter()
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

    pub fn delete_way(&mut self, key: WayKey) -> Result<Way, ObjectDeleteError> {
        let way = self.ways.get(key).ok_or(ObjectDeleteError::NotPresent)?;
        if !way.formed_areas().is_empty() {
            Err(ObjectDeleteError::ContainedInArea)
        } else if !way.containing_relations().is_empty() {
            Err(ObjectDeleteError::ContainedInRelation)
        } else {
            for &node_key in way.nodes() {
                self.nodes[node_key].remove_containing_way(key);
            }
            Ok(self
                .ways
                .remove(key)
                .expect("already checked that the object is present"))
        }
    }

    pub fn iter_ways(&self) -> impl Iterator<Item = (WayKey, &Way)> {
        self.ways.iter()
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
        self.insert_area(Area::new(None, AreaSource::None, rings))
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

    pub fn delete_area(&mut self, key: AreaKey) -> Result<Area, ObjectDeleteError> {
        let area = self.areas.get(key).ok_or(ObjectDeleteError::NotPresent)?;
        if !area.containing_relations().is_empty() {
            Err(ObjectDeleteError::ContainedInRelation)
        } else {
            for ring in area.rings() {
                for &way_key in &ring.ways {
                    self.ways[way_key].remove_formed_area(key);
                }
            }
            Ok(self
                .areas
                .remove(key)
                .expect("already checked that the object is present"))
        }
    }

    pub fn iter_areas(&self) -> impl Iterator<Item = (AreaKey, &Area)> {
        self.areas.iter()
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

    pub fn delete_relation(&mut self, key: RelationKey) -> Result<Relation, ObjectDeleteError> {
        let relation = self
            .relations
            .get(key)
            .ok_or(ObjectDeleteError::NotPresent)?;
        if !relation.containing_relations().is_empty() {
            Err(ObjectDeleteError::ContainedInRelation)
        } else {
            for member in &relation.members().to_vec() {
                self[member.object].remove_containing_relation(key);
            }
            Ok(self
                .relations
                .remove(key)
                .expect("already checked that the object is present"))
        }
    }

    pub fn iter_relations(&self) -> impl Iterator<Item = (RelationKey, &Relation)> {
        self.relations.iter()
    }

    pub fn iter_objects(&self) -> impl Iterator<Item = (ObjectKey, &dyn Object)> {
        let nodes = self
            .iter_nodes()
            .map(|(node_key, node)| (ObjectKey::Node(node_key), node as &dyn Object));
        let ways = self
            .iter_ways()
            .map(|(way_key, way)| (ObjectKey::Way(way_key), way as &dyn Object));
        let areas = self
            .iter_areas()
            .map(|(area_key, area)| (ObjectKey::Area(area_key), area as &dyn Object));
        let relations = self.iter_relations().map(|(relation_key, relation)| {
            (ObjectKey::Relation(relation_key), relation as &dyn Object)
        });
        nodes.chain(ways).chain(areas).chain(relations)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectDeleteError {
    NotPresent,
    ContainedInWay,
    ContainedInArea,
    ContainedInRelation,
}

impl ObjectDeleteError {
    pub fn is_contained_in_object(self) -> bool {
        match self {
            Self::NotPresent => false,
            Self::ContainedInWay | Self::ContainedInArea | Self::ContainedInRelation => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::area::RingRole;

    use super::*;

    fn create_storage() -> Storage {
        Storage::new(Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap())
    }

    fn assert_same_elements<T: PartialEq + Debug>(left: &[T], right: &[T]) {
        assert_eq!(left.len(), right.len(), "lengths do not match");
        for left_element in left {
            let left_count = left.iter().filter(|x| *x == left_element).count();
            let right_count = right.iter().filter(|x| *x == left_element).count();
            assert_eq!(
                left_count, right_count,
                "counts for element {:?} do not match",
                left_element
            );
        }
    }

    #[test]
    fn object_insertion_and_manipulation() {
        let mut storage = create_storage();
        const NODE_COUNT: usize = 4;
        let node_locations = (0..4)
            .map(|i| Location::new(111111.0 * i as f64, 222222.0 * i as f64))
            .collect::<Vec<_>>();
        let node_keys = node_locations
            .iter()
            .enumerate()
            .map(|(i, location)| storage.insert_node(Node::new(Some(i as i64), *location)))
            .collect::<Vec<_>>();
        for i in 0..NODE_COUNT {
            let key = node_keys[i];
            assert!(storage.contains_node(key));
            let node = &storage[key];
            assert_eq!(node.location(), node_locations[i]);
            assert_eq!(node.osm_id(), Some(i as i64));
        }

        let way0 = Way::new(
            Some(10),
            vec![node_keys[0], node_keys[3], node_keys[1], node_keys[2]],
        );
        let way0_key = storage.insert_way(way0);
        assert!(storage.contains_way(way0_key));
        storage.remove_node_from_way(way0_key, 1);
        assert_eq!(
            storage[way0_key].nodes(),
            vec![node_keys[0], node_keys[1], node_keys[2]]
        );

        let way1 = Way::new(Some(11), vec![]);
        let way1_key = storage.insert_way(way1);
        storage.append_node_to_way(way1_key, node_keys[3]);
        storage.insert_node_to_way(way1_key, node_keys[2], 0);
        storage.insert_node_to_way(way1_key, node_keys[0], 2);
        assert_eq!(
            storage[way1_key].nodes(),
            vec![node_keys[2], node_keys[3], node_keys[0]]
        );

        let way2 = Way::new(
            Some(12),
            vec![node_keys[0], node_keys[1], node_keys[2], node_keys[0]],
        ); // closed way
        let way2_key = storage.insert_way(way2);

        assert_same_elements(
            storage[node_keys[0]].containing_ways(),
            &[way0_key, way1_key, way2_key, way2_key],
        );
        assert_same_elements(
            storage[node_keys[1]].containing_ways(),
            &[way0_key, way2_key],
        );
        assert_same_elements(
            storage[node_keys[2]].containing_ways(),
            &[way0_key, way1_key, way2_key],
        );
        assert_same_elements(storage[node_keys[3]].containing_ways(), &[way1_key]);

        let area = Area::new(
            Some(20),
            AreaSource::None,
            vec![Ring::new_inner(vec![way0_key, way1_key])],
        );
        let area_key = storage.insert_area(area);
        assert!(storage.contains_area(area_key));

        storage.remove_ring_from_area(area_key, 0);
        storage.add_ring_to_area(area_key, Ring::new_outer(vec![way0_key, way1_key]));
        assert_same_elements(
            storage[area_key].rings(),
            &[Ring {
                role: RingRole::Outer,
                ways: vec![way0_key, way1_key],
            }],
        );

        assert_same_elements(storage[way0_key].formed_areas(), &[area_key]);
        assert_same_elements(storage[way1_key].formed_areas(), &[area_key]);

        let relation = Relation::new(
            Some(30),
            vec![
                RelationMember::new(node_keys[0], Some("first".into())),
                RelationMember::new(way0_key, None),
                RelationMember::new(area_key, Some("area".into())),
            ],
        );
        let relation_key = storage.insert_relation(relation);
        assert!(storage.contains_relation(relation_key));

        storage.insert_member_to_relation(
            relation_key,
            RelationMember::new(area_key, Some("also area".into())),
            1,
        );
        storage.append_member_to_relation(
            relation_key,
            RelationMember::new(relation_key, Some("itself".into())),
        );
        storage.remove_member_from_relation(relation_key, 0);

        assert_eq!(
            storage[relation_key].members(),
            &[
                RelationMember {
                    object: ObjectKey::Area(area_key),
                    role: Some("also area".into()),
                },
                RelationMember {
                    object: ObjectKey::Way(way0_key),
                    role: None,
                },
                RelationMember {
                    object: ObjectKey::Area(area_key),
                    role: Some("area".into()),
                },
                RelationMember {
                    object: ObjectKey::Relation(relation_key),
                    role: Some("itself".into()),
                },
            ],
        );

        assert_same_elements(storage[node_keys[0]].containing_relations(), &[]);
        assert_same_elements(storage[way0_key].containing_relations(), &[relation_key]);
        assert_same_elements(
            storage[area_key].containing_relations(),
            &[relation_key, relation_key],
        );
        assert_same_elements(
            storage[relation_key].containing_relations(),
            &[relation_key],
        );
    }

    #[test]
    fn object_iteration() {
        let mut storage = create_storage();
        let node_key = storage.insert_node(Node::new(Some(1), Location::new(12345.0, 67890.0)));
        let way_key = storage.insert_way(Way::new(Some(2), vec![]));
        let area_key = storage.insert_area(Area::new(Some(3), AreaSource::None, vec![]));
        let relation_key = storage.insert_relation(Relation::new(Some(4), vec![]));

        assert_same_elements(
            &storage.iter_nodes().collect::<Vec<_>>(),
            &[(node_key, &storage[node_key])],
        );
        assert_same_elements(
            &storage.iter_ways().collect::<Vec<_>>(),
            &[(way_key, &storage[way_key])],
        );
        assert_same_elements(
            &storage.iter_areas().collect::<Vec<_>>(),
            &[(area_key, &storage[area_key])],
        );
        assert_same_elements(
            &storage.iter_relations().collect::<Vec<_>>(),
            &[(relation_key, &storage[relation_key])],
        );
        assert_same_elements(
            &storage
                .iter_objects()
                .map(|(key, obj)| (key, obj.osm_id()))
                .collect::<Vec<_>>(),
            &[
                (ObjectKey::Node(node_key), Some(1)),
                (ObjectKey::Way(way_key), Some(2)),
                (ObjectKey::Area(area_key), Some(3)),
                (ObjectKey::Relation(relation_key), Some(4)),
            ],
        );
    }

    #[test]
    fn object_deletion() {
        let mut storage = create_storage();
        let node_key = storage.insert_node(Node::new(Some(0), Location::new(12345.0, 67890.0)));
        let way_key = storage.insert_way(Way::new(Some(1), vec![node_key]));
        assert_eq!(
            storage.delete_node(node_key),
            Err(ObjectDeleteError::ContainedInWay)
        );
        assert_eq!(
            storage.delete_way(way_key),
            Ok(Way::new(Some(1), vec![node_key]))
        );
        assert_eq!(storage.iter_nodes().count(), 1);
        assert!(storage.contains_node(node_key));
        assert_eq!(storage.iter_ways().count(), 0);
        assert!(!storage.contains_way(way_key));
        assert!(storage[node_key].containing_ways().is_empty());
    }

    #[test]
    #[should_panic(expected = "invalid SlotMap key used")]
    fn deleted_key_access() {
        let mut storage = create_storage();
        let node_key = storage.create_node(Location::new(111.0, 222.0));
        assert!(storage.delete_node(node_key).is_ok());
        storage.create_node(Location::new(333.0, 444.0));
        let _ = storage[node_key];
    }
}
