use super::flit::{Flit, FlitLoader, FlitMaker, FlitType, Id};
use super::packet::Packet;

use crate::efuse::{
    Efuse, LOCALNET_DOWNLEFT, LOCALNET_DOWNRIGHT, LOCALNET_UPLEFT, LOCALNET_UPRIGHT,
};
use rand::Rng;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LocalNetworkLocation {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug)]
pub struct LocalNetwork {
    /// relative location in the same localnet
    location: LocalNetworkLocation,
    /// share in the same localnet
    localnet_id: Id,
    /// it is about global network. so we read efuse value.
    is_root: bool,
    /// neighbor ids
    others_ids: [Id; 3],
    /// mac address
    mac_address: Id,
}

impl LocalNetwork {
    /// LocalNetwork is a 2x2 network.
    /// Basicly, localnetwork information is stored in efuse.
    /// master of localnet is upleft.
    /// others are slaves.
    pub fn new() -> LocalNetwork {
        let efuse = Efuse::new();
        let location: LocalNetworkLocation = efuse.efuse_to_localnet();
        let localnet_id = efuse.get_localnet_id() as Id;
        let mac_address = efuse.get_mac_address() as Id;
        let is_root = efuse.is_root();

        // localnet nodes' form is like this:
        // 0x00000000 [ localnet_id ] [ location ] [ is_root ]
        let others_ids = {
            let raw_localnet = efuse.get_raw_localnet_id();
            let raw_location = efuse.get_raw_localnet_location();
            let raw_is_root = efuse.get_raw_root();

            let mut ids = [0; 3];
            let mut index = 0;
            for raw_other_location in (0..8).step_by(2) {
                if raw_location == raw_other_location {
                    continue;
                }
                let id = raw_localnet | raw_other_location | raw_is_root;
                ids[index] = id as Id;
                index += 1;
            }
            ids
        };

        LocalNetwork {
            location,
            localnet_id,
            is_root,
            others_ids,
            mac_address,
        }
    }

    // ------------------------------
    // only for root
    // ------------------------------

    pub fn root_coordinate(&self) -> (Id, Id) {
        match &self.location {
            LocalNetworkLocation::UpLeft => (0, 1),
            LocalNetworkLocation::UpRight => (1, 1),
            LocalNetworkLocation::DownLeft => (0, 0),
            LocalNetworkLocation::DownRight => (1, 0),
        }
    }

    // ------------------------------
    // getter
    // ------------------------------

    pub fn get_mac_address(&self) -> Id {
        self.mac_address
    }
    pub fn get_localnet_id(&self) -> Id {
        self.localnet_id
    }
    pub fn is_root(&self) -> bool {
        self.is_root
    }
    pub fn get_location(&self) -> LocalNetworkLocation {
        self.location
    }
    pub fn get_others_ids(&self) -> [Id; 3] {
        self.others_ids
    }
}

// test
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        // root and upleft
        let mut block3 = [0; 8];
        block3[7] = 0x00000001;
        let efuse = Efuse { block3 };

        let localnet = LocalNetwork::new();
        println!("{:?}", localnet);
    }
}
