mod flit;
mod flitbuffer;
mod header;
pub mod localnet;
mod packet;
pub mod protocol;
mod sender;

use std::{thread::sleep, time::Duration};

use crate::serial::Serial;
use anyhow::Result;

use flit::{Coordinate, Flit, Id};
use localnet::LocalNetwork;
use packet::Packet;
use protocol::Protocol;

use self::{
    flit::FlitMaker,
    header::Header,
    localnet::LocalNetworkLocation,
    packet::{PacketId, ToId},
    sender::Sender,
};

pub struct NetworkNode<'d> {
    /// now, mac address of localnet is used as node_id (in short, ip address = mac address)
    ip_address: Id,
    /// use vec because it has just 4 elements in most cases so I think hashmap is not needed
    neighbor_localnet_id_and_coordinate: Vec<(Id, Coordinate)>,
    localnet: LocalNetwork,
    coordinate: Coordinate,
    serial: Serial<'d>,

    // for packet
    packet_id: PacketId,
}

impl<'d> NetworkNode<'d> {
    pub fn new(serial: Serial<'d>) -> Self {
        let localnet = LocalNetwork::new();
        let neighbor_in_localnet: Vec<Id> = localnet.get_neighbor_ids().into();

        if localnet.is_root() {
            let mut neighbor_localnet_id_and_coordinate: Vec<(Id, Coordinate)> = Vec::new();
            for localnet_id in neighbor_in_localnet.iter() {
                let location = LocalNetworkLocation::from_id(*localnet_id);
                let coordinate = location.get_root_coordinate();
                neighbor_localnet_id_and_coordinate.push((*localnet_id, coordinate));
            }
            return NetworkNode {
                ip_address: localnet.get_mac_address(),
                neighbor_localnet_id_and_coordinate,
                coordinate: localnet.root_coordinate(),
                localnet,
                serial,

                packet_id: 0,
            };
        }
        let mut neighbor_confirmed: Vec<(Id, Coordinate)> = Vec::new();

        // todo
        while !Self::is_ready(&serial, &mut neighbor_confirmed, localnet.get_mac_address()) {
            continue;
        }
        let coordinate = localnet.get_coordinate(&serial);
        NetworkNode {
            ip_address: localnet.get_mac_address(),
            neighbor_localnet_id_and_coordinate: Vec::new(),
            localnet,
            coordinate,
            serial,

            packet_id: 0,
        }
    }
    /// This function has two roles.
    /// Firstly, send broadcast flit and receive coordinate of neighbor nodes.
    /// Secondly, periodically send flit to neighbor which is in localnet,
    /// and check whether it has received flit of coordinate from neighbor which is not in localnet.
    pub fn is_ready(
        serial: &Serial,
        neighbor_confirmed: &mut Vec<(Id, Coordinate)>,
        source_id: Id,
    ) -> Result<bool> {
        let flit =
            FlitMaker::make_head_flit(0, Header::RequestConfirmedCoordinate, source_id, 0xffff, 0);
        flit.send_broadcast(serial);
        // delay
        sleep(Duration::from_millis(100));
        let received_flit = Flit::receive(serial)?;
        if received_flit.get_header() == Header::ResponseConfirmedCoordinate {
            let coordinate = received_flit.get_coordinate();
            neighbor_confirmed.push((received_flit.get_from(), coordinate));
            return Ok(true);
        }
        sleep(Duration::from_millis(100));
        // let flit = FlitMaker::make_head_flit(
        unimplemented!();
    }

    pub fn is_root(&self) -> bool {
        self.localnet.is_root()
    }

    pub fn make_packet(
        &mut self,
        header: Header,
        from: Id,
        to: ToId,
        messages: Vec<u8>,
    ) -> Result<Packet> {
        let packet = Packet::new(self.packet_id, header, from, to, messages)?;
        self.packet_id += 1;
        Ok(packet)
    }

    #[allow(dead_code)]
    pub fn send_string(&mut self, string: &str) -> Result<()> {
        let packet = self.make_packet(
            Header::Data,
            self.ip_address,
            ToId::Broadcast,
            string.as_bytes().to_vec(),
        )?;
        packet.send_broadcast(&self.serial)?;
        Ok(())
    }

    pub fn get_packet<T>(&self, protocol: &T) -> Result<Option<Packet>>
    where
        T: Protocol,
    {
        // whether there is data in buffer.
        let packet = match Packet::receive(&self.serial) {
            Ok(packet) => packet,
            Err(_) => {
                // unrecovered error
                // maybe, should use flush function
                return Err(anyhow::anyhow!("unrecovered error"));
            }
        };
        // whether it is packet that was sent to this node
        // if it is not, look routing table and send it to next node or not.
        let to = packet.get_to();
        let from = packet.get_from();
        match to {
            ToId::Broadcast => {}
            ToId::Unicast(id) => {
                // unicast
                // send to specific node
                if id != self.ip_address {
                    // send to next node
                    if protocol.is_in_route(self.ip_address, from, id) {
                        // send to next node
                        // todo: this is not efficient way because you just need to change head
                        // flit.
                        let mut packet = packet;
                        packet.change_from(self.ip_address);
                        packet.send(&self.serial, protocol)?;
                    }

                    return Ok(None);
                }
            }
        }
        // it is my packet
        Ok(Some(packet))
    }
    pub fn send_packet<T>(&mut self, packet: Packet, protocol: &T) -> Result<()>
    where
        T: Protocol,
    {
        packet.send(&self.serial, protocol)?;
        Ok(())
    }
    pub fn print_coordinate(&self) {
        println!("coordinate: {:?}", self.coordinate);
    }
}
