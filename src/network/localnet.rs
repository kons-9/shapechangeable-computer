use crate::efuse::Efuse;
use crate::id_utils::{
    type_alias::{Coordinate, Id},
    util,
    util_const::*,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LocalNetworkLocation {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl LocalNetworkLocation {
    pub fn diagonal_location(&self) -> LocalNetworkLocation {
        match self {
            LocalNetworkLocation::UpLeft => LocalNetworkLocation::DownRight,
            LocalNetworkLocation::UpRight => LocalNetworkLocation::DownLeft,
            LocalNetworkLocation::DownLeft => LocalNetworkLocation::UpRight,
            LocalNetworkLocation::DownRight => LocalNetworkLocation::UpLeft,
        }
    }
    pub fn get_root_coordinate(&self) -> Coordinate {
        match &self {
            LocalNetworkLocation::UpLeft => (0, 1),
            LocalNetworkLocation::UpRight => (1, 1),
            LocalNetworkLocation::DownLeft => (0, 0),
            LocalNetworkLocation::DownRight => (1, 0),
        }
    }
    pub fn from_id(id: Id) -> Self {
        let id = (id & 0b110) as Id;
        match id {
            LOCALNET_UPLEFT => LocalNetworkLocation::UpLeft,
            LOCALNET_UPRIGHT => LocalNetworkLocation::UpRight,
            LOCALNET_DOWNLEFT => LocalNetworkLocation::DownLeft,
            LOCALNET_DOWNRIGHT => LocalNetworkLocation::DownRight,
            _ => panic!("Invalid localnet: localnet is less than 5, but {}", id),
        }
    }
    pub fn rotate_clockwise(&self) -> Self {
        match self {
            LocalNetworkLocation::UpLeft => LocalNetworkLocation::UpRight,
            LocalNetworkLocation::UpRight => LocalNetworkLocation::DownRight,
            LocalNetworkLocation::DownLeft => LocalNetworkLocation::UpLeft,
            LocalNetworkLocation::DownRight => LocalNetworkLocation::DownLeft,
        }
    }
    pub fn rotate_counterclockwise(&self) -> Self {
        match self {
            LocalNetworkLocation::UpLeft => LocalNetworkLocation::DownLeft,
            LocalNetworkLocation::UpRight => LocalNetworkLocation::UpLeft,
            LocalNetworkLocation::DownLeft => LocalNetworkLocation::DownRight,
            LocalNetworkLocation::DownRight => LocalNetworkLocation::UpRight,
        }
    }
}
impl From<LocalNetworkLocation> for Id {
    fn from(value: LocalNetworkLocation) -> Self {
        match value {
            LocalNetworkLocation::UpLeft => LOCALNET_UPLEFT,
            LocalNetworkLocation::UpRight => LOCALNET_UPRIGHT,
            LocalNetworkLocation::DownLeft => LOCALNET_DOWNLEFT,
            LocalNetworkLocation::DownRight => LOCALNET_DOWNRIGHT,
        }
    }
}
impl From<Id> for LocalNetworkLocation {
    fn from(value: Id) -> Self {
        match value {
            LOCALNET_UPLEFT => LocalNetworkLocation::UpLeft,
            LOCALNET_UPRIGHT => LocalNetworkLocation::UpRight,
            LOCALNET_DOWNLEFT => LocalNetworkLocation::DownLeft,
            LOCALNET_DOWNRIGHT => LocalNetworkLocation::DownRight,
            _ => panic!("Invalid localnet: localnet is less than 5, but {}", value),
        }
    }
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
    neighbor_ids: [Id; 2],
    diagonal_id: Id,
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
        let mac_address = efuse.get_mac_address();
        let location: LocalNetworkLocation = util::get_localnet_location(mac_address);
        let localnet_id = util::get_localnet_id(mac_address);
        let is_root = util::is_root(mac_address);

        // localnet nodes' form is like this:
        // 0x00000000 [ localnet_id ] [ location ] [ is_root ]
        let (neighbor_ids, diagonal_id) = {
            let raw_localnet = util::get_raw_localnet_id(mac_address);
            let raw_location = util::get_raw_localnet_location(mac_address);
            let raw_is_root = util::get_raw_root(mac_address);

            let diagonal_location = location.diagonal_location();
            let raw_diagonal_location: Id = diagonal_location.into();
            let diagonal_id = raw_localnet | raw_diagonal_location | raw_is_root;

            let mut ids = [0; 2];
            let mut index = 0;
            for raw_other_location in (0..8).step_by(2) {
                if raw_location == raw_other_location {
                    continue;
                }
                if raw_diagonal_location == raw_other_location {
                    continue;
                }
                let id = raw_localnet | raw_other_location | raw_is_root;
                ids[index] = id as Id;
                index += 1;
            }
            (ids, diagonal_id)
        };

        LocalNetwork {
            location,
            localnet_id,
            is_root,
            neighbor_ids,
            diagonal_id,

            mac_address,
        }
    }
    pub fn get_nodes_in_localnet(&self) -> Vec<Id> {
        let mut ids = Vec::new();
        for node in self.neighbor_ids.iter() {
            ids.push(*node);
        }
        ids.push(self.diagonal_id);
        ids
    }

    // ------------------------------
    // only for root
    // ------------------------------

    pub fn root_coordinate(&self) -> Coordinate {
        self.location.get_root_coordinate()
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
    pub fn get_neighbor_ids(&self) -> [Id; 2] {
        self.neighbor_ids
    }
    pub fn get_diagonal_id(&self) -> Id {
        self.diagonal_id
    }
}

// test
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_root() {
        // root and upleft
        let mut block3 = [0; 8];
        block3[7] = 0x00000001;
        // let efuse = Efuse { block3 };

        let localnet = LocalNetwork::new();
        println!("{:?}", localnet);
    }
}
