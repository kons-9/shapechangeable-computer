pub mod flit;
pub mod header;
pub mod localnet;
pub mod packet;
pub mod protocol;
pub mod serial;
pub mod system;
pub mod utils;

use std::{thread::sleep, time::Duration};

use crate::{
    serial::SerialTrait,
    utils::util::{add_x, add_y, calculate_l0_distance, is_same_localnet},
};
use system::SystemInfo;
use utils::type_alias::{Coordinate, Id};

use anyhow::{anyhow, Result};
use log::info;

use localnet::LocalNetwork;
use packet::Packet;
pub use protocol::Protocol;

use self::{
    header::Header,
    localnet::LocalNetworkLocation,
    packet::{PacketId, ToId},
};

pub struct NetworkNode<T, S>
where
    T: Protocol,
    S: SerialTrait,
{
    /// now, mac address of localnet is used as node_id (in short, ip address = mac address)
    ip_address: Id,
    localnet: LocalNetwork,
    global_location: LocalNetworkLocation,
    coordinate: Coordinate,
    serial: S,
    protocol: T,

    // for packet
    packet_id: PacketId,
}

impl<T, S> NetworkNode<T, S>
where
    T: Protocol,
    S: SerialTrait,
{
    pub fn new(mut serial: S, protocol: T, system_info: &impl SystemInfo) -> Result<Self> {
        let localnet = LocalNetwork::new(system_info);
        let neighbor_in_localnet: Vec<Id> = localnet.get_neighbor_ids().into();

        if localnet.is_root() {
            info!("root node");
            let mut localnet_id_and_coordinate: Vec<(Id, Coordinate)> = Vec::new();
            for localnet_id in neighbor_in_localnet.iter() {
                let location = LocalNetworkLocation::from_id(*localnet_id);
                let coordinate = location.get_root_coordinate();
                localnet_id_and_coordinate.push((*localnet_id, coordinate));
            }
            info!(
                "localnet_id_and_coordinate: {:?}",
                localnet_id_and_coordinate
            );
            // same as locallocation
            let global_location = localnet.get_location();
            info!("global_location: {:?}", global_location);
            return Ok(NetworkNode {
                ip_address: localnet.get_mac_address(),
                coordinate: localnet.root_coordinate(),
                localnet,
                global_location,
                serial,
                protocol,

                packet_id: 0,
            });
        }
        // ///////////////////////////////////////////
        // not root node
        // //////////////////////////////////////////

        info!("not root node");
        let ip_address = localnet.get_mac_address();

        // not root, so need to connect other nodes(units).
        let mut neighbor_confirmed: Vec<(Id, Coordinate)> = Vec::new();

        'outer: loop {
            loop {
                match Self::check_connection(&mut serial, ip_address) {
                    Ok(true) => break,
                    Ok(false) => {
                        info!("connection is not exist");
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                        serial.flush_all()?;
                    }
                }
                std::thread::sleep(Duration::from_millis(3000));
                continue;
            }
            info!("confirmed connection with other nodes");

            while !Self::is_ready(&neighbor_confirmed, ip_address)? {
                // send broadcast packet
                match Self::request_confirmed_coordinate(&mut serial, ip_address) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {:?}", e);
                        serial.flush_all()?;
                        continue;
                    }
                }

                // delay
                sleep(Duration::from_millis(3000));

                let mut loop_count = 0;
                loop {
                    if loop_count > 10000 {
                        // time out
                        continue 'outer;
                    }
                    let received_packet = match Packet::receive(&mut serial) {
                        Ok(Some(packet)) => packet,
                        Ok(None) => {
                            sleep(Duration::from_millis(10));
                            loop_count += 1;
                            continue;
                        }
                        Err(e) => {
                            println!("error: {:?}", e);
                            loop_count += 1;
                            serial.flush_all()?;
                            continue;
                        }
                    };
                    let is_valid = Self::process_received_packet_of_request(
                        &mut serial,
                        ip_address,
                        received_packet,
                        &mut neighbor_confirmed,
                    )?;

                    if !is_valid {
                        serial.flush_read()?;
                        info!("unexpected packet");
                        loop_count += 1;
                        continue;
                    }
                    break 'outer;
                }
            }
        }

        let (coordinate, global_location) =
            Self::coordinate_and_global_location_from_neighbor_confirmed(
                &neighbor_confirmed,
                ip_address,
            )?;

        // todo: Join global network

        Ok(NetworkNode {
            ip_address,
            localnet,
            global_location,
            coordinate,
            serial,
            protocol,

            packet_id: 1,
        })
    }
    fn request_confirmed_coordinate(serial: &mut S, node_id: Id) -> Result<()> {
        let packet = Packet::make_request_confirmed_coordinate_packet(node_id);
        packet.send(serial)?;
        Ok(())
    }
    fn process_received_packet_of_request(
        serial: &mut S,
        node_id: Id,
        received_packet: Packet,
        neighbor_confirmed: &mut Vec<(Id, Coordinate)>,
    ) -> Result<bool> {
        match received_packet.get_header() {
            Header::ConfirmCoordinate => {
                // if received packed source node is in the same localnet of this node,
                let coordinates = received_packet.load_confirmed_coordinate_packet(node_id)?;

                for coordinate in coordinates {
                    neighbor_confirmed.push(coordinate);
                }
                return Ok(true);
            }
            Header::HRequestConfirmedCoordinate => {
                if neighbor_confirmed.len() != 0 {
                    let packet = Packet::make_reply_for_request_confirmed_coordinate_packet(
                        node_id,
                        received_packet.get_global_from(),
                        neighbor_confirmed,
                        None,
                    )?;
                    packet.send(serial)?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn coordinate_and_global_location_from_neighbor_confirmed(
        neighbor_confirmed: &Vec<(Id, Coordinate)>,
        this_id: Id,
    ) -> Result<(Coordinate, LocalNetworkLocation)> {
        let (coordinate, id_cmp, coordinate_cmp) =
            match Self::find_distance_1_neighbor(neighbor_confirmed) {
                Some((_, coordinate, id_cmp, coordinate_cmp)) => {
                    (coordinate, id_cmp, coordinate_cmp)
                }
                None => {
                    return Err(anyhow::anyhow!("not found distance 1 neighbor"));
                }
            };
        let location = LocalNetworkLocation::from_id(this_id);
        let location_cmp = LocalNetworkLocation::from_id(id_cmp);
        return Ok(
            Self::get_global_coordinate_and_global_location_from_local_location(
                location,
                coordinate,
                location_cmp,
                coordinate_cmp,
            ),
        );
    }
    fn get_global_coordinate_and_global_location_from_local_location(
        local_location: LocalNetworkLocation,
        coordinate: Coordinate,
        local_location_cmp: LocalNetworkLocation,
        coordinate_cmp: Coordinate,
    ) -> (Coordinate, LocalNetworkLocation) {
        let is_clockwise_location = if local_location.rotate_clockwise() == local_location_cmp {
            true
        } else if local_location.rotate_counterclockwise() == local_location_cmp {
            false
        } else {
            unreachable!();
        };
        const X: bool = true;
        const Y: bool = false;
        let different_coordinate = if coordinate.0 != coordinate_cmp.0 {
            X
        } else if coordinate.1 != coordinate_cmp.1 {
            Y
        } else {
            unreachable!();
        };
        let is_small_coordinate =
            if coordinate.0 < coordinate_cmp.0 || coordinate.1 < coordinate_cmp.1 {
                true
            } else {
                false
            };
        match (
            is_clockwise_location,
            different_coordinate,
            is_small_coordinate,
        ) {
            (true, X, true) => (add_y(coordinate, -1), LocalNetworkLocation::UpLeft),
            (true, X, false) => (add_y(coordinate, 1), LocalNetworkLocation::DownRight),
            (true, Y, true) => (add_x(coordinate, 1), LocalNetworkLocation::DownLeft),
            (true, Y, false) => (add_x(coordinate, -1), LocalNetworkLocation::UpRight),
            (false, X, true) => (add_y(coordinate, -1), LocalNetworkLocation::UpRight),
            (false, X, false) => (add_y(coordinate, 1), LocalNetworkLocation::DownLeft),
            (false, Y, true) => (add_x(coordinate, -1), LocalNetworkLocation::DownRight),
            (false, Y, false) => (add_x(coordinate, 1), LocalNetworkLocation::UpLeft),
        }
    }
    fn find_distance_1_neighbor(
        neighbor_confirmed: &Vec<(Id, Coordinate)>,
    ) -> Option<(Id, Coordinate, Id, Coordinate)> {
        for i in 0..neighbor_confirmed.len() {
            let (id, coordinate) = neighbor_confirmed[i];
            for j in i + 1..neighbor_confirmed.len() {
                let (id_cmp, coordinate_cmp) = neighbor_confirmed[j];
                // calculate distance between coordinate and coordinate_cmp
                let distance = calculate_l0_distance(coordinate, coordinate_cmp);
                if distance == 1 {
                    return Some((id, coordinate, id_cmp, coordinate_cmp));
                }
            }
        }
        None
    }
    pub fn join_global_network(&mut self) {
        // todo:
        ()
    }

    /// check connection with other nodes that is not in the same local network.
    fn check_connection(serial: &mut S, node_id: Id) -> Result<bool> {
        info!("making check connection packet");
        let packet = Packet::make_check_connection_packet(node_id);
        packet.send(serial)?;
        info!("send check connection packet");
        let received_packet = match Packet::receive(serial)? {
            Some(_packet) => {
                if _packet.get_from() == node_id {
                    return Ok(false);
                }
                _packet
            }
            None => return Ok(false),
        };
        match received_packet.get_header() {
            Header::HCheckConnection => {
                let source = received_packet.get_from();
                if is_same_localnet(node_id, source) {
                    Ok(false)
                } else {
                    info!("received_packet: {:?}", received_packet);
                    Ok(true)
                }
            }
            _ => Ok(false),
        }
    }

    /// This function has two roles.
    /// Firstly, send broadcast flit and receive coordinate of neighbor nodes.
    /// Secondly, periodically send flit to neighbor which is in localnet,
    /// and check whether it has received flit of coordinate from neighbor which is not in localnet.
    fn is_ready(neighbor_confirmed: &Vec<(Id, Coordinate)>, this_id: Id) -> Result<bool> {
        // abviously, not enough
        if neighbor_confirmed.len() < 1 {
            return Ok(false);
        }

        // check whether there is a node that is not in the same local network.
        for (id, _) in neighbor_confirmed.iter() {
            if *id == this_id || !is_same_localnet(*id, this_id) {
                return Err(anyhow!(
                    "software bug: the same id is exist in neighbor_confirmed"
                ));
            }
        }

        Ok(Self::find_distance_1_neighbor(neighbor_confirmed).is_some())
    }

    pub fn is_root(&self) -> bool {
        self.localnet.is_root()
    }

    pub fn make_packet(
        &mut self,
        header: Header,
        globalfrom: Id,
        globalto: ToId,
        messages: Vec<u8>,
    ) -> Result<Packet> {
        let from = self.ip_address;
        let to = self
            .protocol
            .get_next_node(self.ip_address, globalto.to_id());
        let packet = Packet::new(
            self.packet_id,
            header,
            globalfrom,
            globalto,
            from,
            ToId::from_id(to),
            messages,
        );
        self.packet_id += 1;
        Ok(packet)
    }

    pub fn get_messages(&mut self) -> Result<Option<Vec<u8>>> {
        match self.get_packet()? {
            Some(packet) => Ok(Some(packet.get_messages())),
            None => Ok(None),
        }
    }

    /// get packet from serial
    pub fn get_packet(&mut self) -> Result<Option<Packet>> {
        // whether there is data in buffer.
        let packet = match Packet::receive(&mut self.serial) {
            Ok(Some(packet)) => packet,
            Ok(None) => {
                // no data in buffer
                return Ok(None);
            }
            Err(_) => {
                info!("receive error in get_packet");
                self.flush_all()?;
                return self.get_packet();
            }
        };
        // whether it is packet that was sent to this node
        // if it is not, look routing table and send it to next node or not.
        let to = packet.get_global_to();
        let from = packet.get_from();
        match to {
            ToId::Broadcast => {}
            ToId::Unicast(global_destination_id) => {
                // unicast
                // send to specific node
                if global_destination_id != self.ip_address {
                    // send to next node
                    if self
                        .protocol
                        .is_in_route(self.ip_address, from, global_destination_id)
                    {
                        // send to next node
                        // todo: this is not efficient way because you just need to change head
                        // flit.
                        let mut packet = packet;
                        packet.change_from_and_to(
                            self.ip_address,
                            ToId::from_id(
                                self.protocol
                                    .get_next_node(self.ip_address, global_destination_id),
                            ),
                        );
                        packet.send(&mut self.serial)?;
                    }

                    return Ok(None);
                }
            }
        }
        // it is my packet
        Ok(Some(packet))
    }
    pub fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }
    pub fn get_local_location(&self) -> LocalNetworkLocation {
        self.localnet.get_location()
    }
    pub fn get_global_location(&self) -> LocalNetworkLocation {
        self.global_location
    }
    pub fn get_ip_address(&self) -> Id {
        self.ip_address
    }
    pub fn print_coordinate(&self) {
        println!("coordinate: {:?}", self.coordinate);
    }
    pub fn flush_read(&mut self) -> Result<()> {
        self.serial.flush_read()?;
        Ok(())
    }
    pub fn flush_write(&mut self) -> Result<()> {
        self.serial.flush_write()?;
        Ok(())
    }
    pub fn flush_all(&mut self) -> Result<()> {
        self.serial.flush_all()?;
        Ok(())
    }
    pub fn send(&mut self, packet: Packet) -> Result<()> {
        packet.send(&mut self.serial)?;
        Ok(())
    }
}
