mod flit;
mod flitbuffer;
mod header;
pub mod localnet;
mod packet;
pub mod protocol;

use std::{thread::sleep, time::Duration};

use crate::serial::Serial;
use anyhow::Result;

use crate::id_utils::TypeAlias::{Coordinate, Id};
use localnet::LocalNetwork;
use packet::Packet;
use protocol::Protocol;

use self::{
    header::Header,
    localnet::LocalNetworkLocation,
    packet::{PacketId, ToId},
};

pub struct NetworkNode<'d, T>
where
    T: Protocol,
{
    /// now, mac address of localnet is used as node_id (in short, ip address = mac address)
    ip_address: Id,
    /// use vec because it has just 4 elements in most cases.
    /// so I think hashmap is not needed
    neighbor_localnet_id_and_coordinate: Vec<(Id, Coordinate)>,
    localnet: LocalNetwork,
    coordinate: Coordinate,
    serial: Serial<'d>,
    protocol: T,

    // for packet
    packet_id: PacketId,
}

impl<'d, T> NetworkNode<'d, T>
where
    T: Protocol,
{
    pub fn new(serial: Serial<'d>, protocol: T) -> Result<Self> {
        let localnet = LocalNetwork::new();
        let neighbor_in_localnet: Vec<Id> = localnet.get_neighbor_ids().into();

        if localnet.is_root() {
            let mut neighbor_localnet_id_and_coordinate: Vec<(Id, Coordinate)> = Vec::new();
            for localnet_id in neighbor_in_localnet.iter() {
                let location = LocalNetworkLocation::from_id(*localnet_id);
                let coordinate = location.get_root_coordinate();
                neighbor_localnet_id_and_coordinate.push((*localnet_id, coordinate));
            }
            return Ok(NetworkNode {
                ip_address: localnet.get_mac_address(),
                neighbor_localnet_id_and_coordinate,
                coordinate: localnet.root_coordinate(),
                localnet,
                serial,
                protocol,

                packet_id: 0,
            });
        }
        let mut neighbor_confirmed: Vec<(Id, Coordinate)> = Vec::new();

        // todo
        while !(Self::is_ready(
            &serial,
            &mut neighbor_confirmed,
            localnet.get_mac_address(),
            &protocol,
        ))? {
            continue;
        }
        let coordinate = localnet.get_coordinate(&serial);
        Ok(NetworkNode {
            ip_address: localnet.get_mac_address(),
            neighbor_localnet_id_and_coordinate: Vec::new(),
            localnet,
            coordinate,
            serial,
            protocol,

            packet_id: 0,
        })
    }
    /// This function has two roles.
    /// Firstly, send broadcast flit and receive coordinate of neighbor nodes.
    /// Secondly, periodically send flit to neighbor which is in localnet,
    /// and check whether it has received flit of coordinate from neighbor which is not in localnet.
    fn is_ready(
        serial: &Serial,
        neighbor_confirmed: &mut Vec<(Id, Coordinate)>,
        source_id: Id,
        protocol: &T,
    ) -> Result<bool> {
        // send broadcast packet
        let packet = Packet::make_request_confirmed_coordinate_packet(source_id);
        packet.send(serial)?;

        // delay
        sleep(Duration::from_millis(100));

        let mut loop_count = 0;
        loop {
            if loop_count > 100 {
                return Ok(false);
            }
            let received_packet = match Packet::receive(serial)? {
                Some(packet) => packet,
                None => {
                    sleep(Duration::from_millis(10));
                    loop_count += 1;
                    continue;
                }
            };

            match received_packet.get_header() {
                Header::ConfirmCoordinate => {
                    // if received packed source node is in the same localnet of this node,
                    // coordinate may be more than 1, but if not, it is 1. if not 1, it meajjjjjjjj
                    let coordinates =
                        received_packet.load_confirmed_coordinate_packet(source_id)?;

                    for coordinate in coordinates {
                        neighbor_confirmed.push(coordinate);
                    }
                    return Ok(neighbor_confirmed.len() > 1);
                }
                Header::HRequestConfirmedCoordinate => {
                    if neighbor_confirmed.len() != 0 {
                        let coordinate = neighbor_confirmed.iter().map(|(_, c)| c).collect();
                        let packet = Packet::make_reply_for_request_confirmed_coordinate_packet(
                            source_id,
                            received_packet.get_from(),
                            coordinate,
                        )?;
                        packet.send(serial)?;
                    }
                }
                _ => {}
            }
            serial.flush_read()?;
            println!("unexpected packet");
            loop_count = 0;
        }
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

    pub fn get_packet(&self) -> Result<Option<Packet>> {
        // whether there is data in buffer.
        let packet = match Packet::receive(&self.serial) {
            Ok(Some(packet)) => packet,
            Ok(None) => {
                // no data in buffer
                return Ok(None);
            }
            Err(_) => {
                // unrecovered error
                // maybe, should use flush function
                return Err(anyhow::anyhow!("unrecovered error"));
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
                        packet.send(&self.serial)?;
                    }

                    return Ok(None);
                }
            }
        }
        // it is my packet
        Ok(Some(packet))
    }
    pub fn print_coordinate(&self) {
        println!("coordinate: {:?}", self.coordinate);
    }
}
