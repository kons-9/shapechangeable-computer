use network_node::utils::type_alias::Id;
use network_node::Protocol;

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
        Self {}
    }
}
impl Protocol for DefaultProtocol {
    fn is_in_route(&self, this_id: Id, source_id: Id, destination_id: Id) -> bool {
        unimplemented!()
    }
    fn get_next_node(&self, this_id: Id, destination_id: Id) -> Id {
        unimplemented!()
    }
}
