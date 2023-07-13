use crate::id_utils::TypeAlias::Id;

pub trait Protocol {
    fn is_in_route(&self, this_id: Id, from_id: Id, destination_id: Id) -> bool;
    fn get_next_node(&self, this_id: Id, destination_id: Id) -> Id;
}

pub struct DefaultProtocol {
    routing_table: RoutingTable,
}

impl DefaultProtocol {
    pub fn new() -> Self {
        Self {
            routing_table: RoutingTable::new(),
        }
    }
}

struct RoutingTable {
    // ...
}
impl RoutingTable {
    fn new() -> Self {
        unimplemented!()
    }
}
impl Protocol for DefaultProtocol {
    fn is_in_route(&self, this_id: Id, from_id: Id, destination_id: Id) -> bool {
        unimplemented!()
    }
    fn get_next_node(&self, this_id: Id, destination_id: Id) -> Id {
        unimplemented!()
    }
}
