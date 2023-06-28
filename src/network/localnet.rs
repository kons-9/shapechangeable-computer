use crate::efuse::{
    Efuse, LOCALNET_DOWNLEFT, LOCALNET_DOWNRIGHT, LOCALNET_UPLEFT, LOCALNET_UPRIGHT,
};
use crate::packet::Id;
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
}

impl LocalNetwork {
    /// LocalNetwork is a 2x2 network.
    /// Basicly, localnetwork information is stored in efuse.
    /// master of localnet is upleft.
    /// others are slaves.
    pub fn new() -> LocalNetwork {
        let efuse = Efuse::new();
        let location = efuse.efuse_to_localnet();
        let is_root = efuse.is_root();

        // id is random.
        // todo: id must be different from neighbor's localnet id.
        // todo: A node that is appended later will use voltage that is provided by other nodes connected through magnetic field.
        // todo: then it will get id from other nodes but it will accidentally get same id with other localnet nodes.
        // todo: In this perspective, we'd better think not using communication when we get id.
        // todo: maybe we will also use efuse to get id. I am still finding the best way to get id.
        let localnet_id;
        if location == LocalNetworkLocation::UpLeft {
            localnet_id = rand::thread_rng().gen_range(1..10000);
            // send localnet_id to slaves
            Self::send_localnet_id(localnet_id);
        } else {
            // receive localnet_id from master
            localnet_id = Self::receive_localnet_id();
        }

        LocalNetwork {
            location,
            localnet_id,
            is_root,
        }
    }

    fn send_localnet_id(localnet_id: Id) {
        // wait for all slaves until they get localnet_id
        unimplemented!();
    }
    fn receive_localnet_id() -> Id {
        // receive localnet_id from master
        unimplemented!();
    }

    // ------------------------------
    // only for root
    // ------------------------------

    pub fn root_coordinate(&self) -> (u32, u32) {
        match &self.location {
            LocalNetworkLocation::UpLeft => (0, 1),
            LocalNetworkLocation::UpRight => (1, 1),
            LocalNetworkLocation::DownLeft => (0, 0),
            LocalNetworkLocation::DownRight => (1, 0),
        }
    }

    pub fn root_node_id(&self) -> Id {
        match &self.location {
            LocalNetworkLocation::UpLeft => 0,
            LocalNetworkLocation::UpRight => 1,
            LocalNetworkLocation::DownLeft => 2,
            LocalNetworkLocation::DownRight => 3,
        }
    }

    // ------------------------------
    // getter
    // ------------------------------

    pub fn get_id(&self) -> Id {
        self.localnet_id
    }
    pub fn get_location(&self) -> LocalNetworkLocation {
        self.location
    }
    pub fn is_root(&self) -> bool {
        self.is_root
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
