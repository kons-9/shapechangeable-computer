use crate::utils::type_alias::Id;

pub trait Protocol {
    /// check whether this node is in route
    fn is_in_route(&self, this: Id, global_source: Id, global_destination: Id) -> bool;
    /// get next node
    fn get_next_node(&self, this: Id, global_destination: Id) -> Id;
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub struct RoutingTable {}
    impl RoutingTable {
        pub fn new() -> Self {
            Self {}
        }
    }

    pub struct TestProtocol {
        routing_table: RoutingTable,
    }

    impl TestProtocol {
        pub fn new() -> Self {
            Self {
                routing_table: RoutingTable::new(),
            }
        }
    }
    impl Protocol for TestProtocol {
        fn is_in_route(&self, this_id: Id, source_id: Id, destination_id: Id) -> bool {
            true
        }
        fn get_next_node(&self, this_id: Id, destination_id: Id) -> Id {
            0
        }
    }
}
