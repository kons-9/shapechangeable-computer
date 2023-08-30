use anyhow::Result;
use network_node::protocol::Protocol;
use network_node::utils::type_alias::Id;

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
        true
    }
    fn get_next_node(&self, this_id: Id, destination_id: Id) -> Id {
        0
    }
    fn add_connection(&mut self, id: Id, id2: Id) -> Result<()> {
        Ok(())
    }
    fn remove_connection(&mut self, id: Id, id2: Id) -> Result<()> {
        Ok(())
    }
}
