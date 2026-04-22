#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    Map(i64),
    Custom(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WayId {
    Map(i64),
    Custom(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AreaId {
    Way(i64),
    Relation(i64),
    Custom(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RelationId {
    Map(i64),
    Custom(i64),
}

#[derive(Clone, Debug)]
pub struct IdProvider {
    next_id: i64,
}

impl IdProvider {
    pub fn new() -> IdProvider {
        IdProvider { next_id: 1 }
    }

    fn advance(&mut self) {
        self.next_id += 1;
    }

    pub fn node(&mut self) -> NodeId {
        let id = NodeId::Custom(self.next_id);
        self.advance();
        id
    }

    pub fn way(&mut self) -> WayId {
        let id = WayId::Custom(self.next_id);
        self.advance();
        id
    }

    pub fn area(&mut self) -> AreaId {
        let id = AreaId::Custom(self.next_id);
        self.advance();
        id
    }

    pub fn relation(&mut self) -> RelationId {
        let id = RelationId::Custom(self.next_id);
        self.advance();
        id
    }
}
