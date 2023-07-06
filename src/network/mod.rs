mod flit;
mod flitbuffer;
mod header;
pub mod localnet;
mod packet;
mod protocol;
mod sender;

use crate::serial::Serial;
use anyhow::Result;

use flit::{Coordinate, Flit, Id};
use localnet::LocalNetwork;
use packet::Packet;

pub struct NetworkNode<'d> {
    /// now, mac address of localnet is used as node_id (in short, ip address = mac address)
    ip_address: Id,
    neighbor_localnet_id_and_coordinate: Vec<(Id, Coordinate)>,
    localnet: LocalNetwork,
    coordinate: Coordinate,
    serial: Serial<'d>,

    // tree structure
    parent: Option<Id>,
    children: Vec<Id>,
}

impl<'d> NetworkNode<'d> {
    pub fn new(serial: Serial<'d>) -> Self {
        let localnet = LocalNetwork::new();

        if localnet.is_root() {
            return NetworkNode {
                ip_address: localnet.get_mac_address(),
                neighbor_localnet_id_and_coordinate: Vec::new(),
                coordinate: localnet.root_coordinate(),
                localnet,
                serial,

                parent: None,
                children: Vec::new(),
            };
        }
        // todo
        let coordinate = (0, 0);
        NetworkNode {
            ip_address: localnet.get_mac_address(),
            neighbor_localnet_id_and_coordinate: Vec::new(),
            localnet,
            coordinate,
            serial,

            parent: None,
            children: Vec::new(),
        }
    }

    pub fn send_string(&mut self, string: &str) -> Result<()> {
        let packet = Packet::new(self.ip_address, string);
        Ok(())
    }

    pub fn try_connect(&mut self) -> Result<Option<(Id, Coordinate)>> {
        if !self.is_connected() {
            Ok(None)
        } else {
            let neightbor_id_coordinate = self.get_neighbor_id_coordinate()?;
            self.neighbor_localnet_id_and_coordinate
                .push(neightbor_id_coordinate);
            Ok(Some(neightbor_id_coordinate))
        }
    }
    pub fn can_calculate_coordinate(&self) -> bool {
        self.neighbor_localnet_id_and_coordinate.len() <= 2
    }

    pub fn calcuate_coordinate(&self) -> Result<Coordinate> {
        if !self.can_calculate_coordinate() {
            return Err(anyhow::anyhow!(
                "Cannot calculate coordinate less than 2 neighbors"
            ));
        }
        for (id, coordinate) in &self.neighbor_localnet_id_and_coordinate {
            unimplemented!();
        }

        Err(anyhow::anyhow!("Unreachable: Invalid neighbor coordinate"))
    }
    pub fn set_coordinate(&mut self, coordinate: Coordinate) {
        self.is_confirmed = true;
        self.coordinate = Some(coordinate);
    }

    pub fn get_id_from_root(&self) -> Result<Id> {
        // after confirned
        unimplemented!();
    }

    fn get_neighbor_id_coordinate(&self) -> Result<(Id, Coordinate)> {
        let localnet_id = self.localnet.get_id();
        // str is "getid_" + localnet_id and type is &[u8]
        let str = format!("getid_{}", localnet_id).as_bytes();

        self.serial.send(&str)?;
        let localnet_neighbor = self.serial.receive()?;
        unimplemented!();
    }

    pub fn is_confirmed(&self) -> bool {
        self.is_confirmed
    }
}
