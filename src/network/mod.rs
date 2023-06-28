mod flit;
mod localnet;
mod packet;

use crate::serial::Serial;
use anyhow::Result;

use localnet::LocalNetwork;
use packet::Coordinate;
use packet::Id;

pub struct Network<'d> {
    node_id: Option<Id>,
    neighbor_localnet_id_and_coordinate: Vec<(Id, Coordinate)>,
    localnet: LocalNetwork,
    coordinate: Option<Coordinate>,
    is_confirmed: bool,
    serial: Serial<'d>,

    // tree structure
    parent: Option<Id>,
    children: Vec<Id>,
}

impl<'d> Network<'d> {
    pub fn new(serial: Serial<'d>) -> Self {
        let localnet = LocalNetwork::new();
        if localnet.is_root() {
            Network {
                node_id: Some(localnet.root_node_id()),
                neighbor_localnet_id_and_coordinate: Vec::new(),
                coordinate: Some(localnet.root_coordinate()),
                localnet,
                is_confirmed: true,
                serial,

                parent: None,
                children: Vec::new(),
            }
        } else {
            Network {
                node_id: None,
                neighbor_localnet_id_and_coordinate: Vec::new(),
                localnet,
                coordinate: None,
                is_confirmed: false,
                serial,

                parent: None,
                children: Vec::new(),
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        unimplemented!();
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
