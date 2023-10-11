use anyhow::anyhow;
use anyhow::Result;
use network_node::protocol::ChannelId;
use network_node::protocol::Protocol;
use network_node::utils::type_alias::Coordinate;
use network_node::utils::type_alias::Id;

// this network is only for 4 * 4 network
pub struct DefaultProtocol {
    routing_table: DefaultRoutingTable,
}
// this routing table is only for 4 * 4 networks
struct DefaultRoutingTable {}
impl DefaultRoutingTable {
    fn get_next_coordinate(
        &self,
        this_coordinate: Coordinate,
        destination_coordinate: Coordinate,
    ) -> Coordinate {
        let (this_x, this_y) = this_coordinate;
        let (destination_x, destination_y) = destination_coordinate;

        // firstly, move x coordinate
        // secondly, move y coordinate
        if this_x < destination_x {
            return (this_x + 1, this_y);
        } else if this_x > destination_x {
            return (this_x - 1, this_y);
        } else if this_y < destination_y {
            return (this_x, this_y + 1);
        } else if this_y > destination_y {
            return (this_x, this_y - 1);
        } else {
            panic!("get_next_coordinate error: this_coordinate == destination_coordinate");
        }
    }
    fn is_in_route(
        &self,
        this_coordinate: Coordinate,
        source_coordinate: Coordinate,
        destination_coordinate: Coordinate,
    ) -> bool {
        let (this_x, this_y) = this_coordinate;
        let (source_x, source_y) = source_coordinate;
        let (destination_x, destination_y) = destination_coordinate;

        // source -> destination path is determined by get_next_coordinate.
        // in the function, firstly, move x coordinate, and secondly, move y coordinate.
        if source_y == this_y && (source_x <= this_x && this_x <= destination_x) {
            true
        } else if destination_x == this_x && (source_y <= this_y && this_y <= destination_y) {
            true
        } else {
            false
        }
    }
}

impl DefaultProtocol {
    pub fn new() -> Self {
        Self {
            routing_table: DefaultRoutingTable {},
        }
    }
    fn make_ip_address(coordinate: Coordinate) -> Result<Id> {
        match coordinate {
            (x, y) if x >= 0 && y >= 0 && x < 4 && y < 4 => return Ok(x as u16 + 4 * y as u16),
            _ => return Err(anyhow!("join_global_net error: not 4 * 4 netowrk")),
        }
    }
    fn get_coordinate(ip_address: Id) -> Result<Coordinate> {
        match ip_address {
            0..=15 => return Ok((ip_address as i16 % 4 as i16, ip_address as i16 / 4)),
            _ => return Err(anyhow!("get_coordinate error: not 4 * 4 netowrk")),
        }
    }
}
impl Protocol for DefaultProtocol {
    // check whether this node is in route
    fn is_in_route(&self, this_id: Id, source_id: Id, destination_id: Id) -> bool {
        self.routing_table.is_in_route(
            Self::get_coordinate(this_id).unwrap(),
            Self::get_coordinate(source_id).unwrap(),
            Self::get_coordinate(destination_id).unwrap(),
        )
    }
    // return next node's ip address
    fn get_next_node(&self, this_id: Id, destination_id: Id) -> Id {
        Self::make_ip_address(self.routing_table.get_next_coordinate(
            Self::get_coordinate(this_id).unwrap(),
            Self::get_coordinate(destination_id).unwrap(),
        ))
        .unwrap()
    }
    // id and id2 are connected
    fn add_connection(&mut self, _id: Id, _id2: Id) -> Result<()> {
        Ok(())
    }
    // id and id2 are disconnected
    fn remove_connection(&mut self, _id: Id, _id2: Id) -> Result<()> {
        Ok(())
    }
    fn get_channel(&self, _this: Id, _destinateion: Id) -> ChannelId {
        0
    }
    fn change_channel(&mut self, _this: Id, _destination: Id, _channel: ChannelId) -> Result<()> {
        Ok(())
    }
    fn join_global_network(&mut self, _mac_address: Id, coordinate: Coordinate) -> Result<Id> {
        Self::make_ip_address(coordinate)
    }
}
