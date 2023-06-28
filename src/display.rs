use crate::localnet::LocalNetwork;

pub struct Display {
    localnet: LocalNetwork,
}

impl Display {
    pub fn new() -> Display {
        let localnet = LocalNetwork::new();
        Display { localnet }
    }
}
