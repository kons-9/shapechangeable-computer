use anyhow::Result;

use crate::utils::type_alias::{Coordinate, Id};

pub type ChannelId = u8;

pub trait Protocol {
    /// check whether this node is in route
    fn is_in_route(&self, this: Id, global_source: Id, global_destination: Id) -> bool;
    /// get next node
    fn get_next_node(&self, this: Id, global_destination: Id) -> Id;
    /// add nodes' connection to routing table
    /// nodes which have id or id2 are connected
    fn add_connection(&mut self, id: Id, id2: Id) -> Result<()>;
    /// remove nodes' connection in routing table
    /// nodes which have id or id2 are disconnected
    fn remove_connection(&mut self, id: Id, id2: Id) -> Result<()>;

    // channels

    ///
    fn get_channel(&self, this: Id, destinateion: Id) -> ChannelId {
        // default implementation has only one channel.
        0
    }
    fn change_channel(&mut self, this: Id, destination: Id, channel: ChannelId) -> Result<()> {
        // default implementation has only one channel, so cannot change it.
        Ok(())
    }
    // return is global network ip_address
    fn join_global_network(&mut self, mac_address: Id, coordinate: Coordinate) -> Result<Id>;
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
        fn add_connection(&mut self, id: Id, id2: Id) -> Result<()> {
            Ok(())
        }
        fn remove_connection(&mut self, id: Id, id2: Id) -> Result<()> {
            Ok(())
        }
    }
}
