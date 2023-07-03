use super::flit::{Coordinate, Flit, FlitLoader, FlitType, Header, Id};
use anyhow::{anyhow, Result};

type From = Id;

// broadcast is represented by 0xFFFF
// localnet is only used when making localnet
enum To {
    Localnet(Id),
    Node(Id),
    Broadcast,
}

struct Packet {
    header: Header,
    from: From,
    to: To,
    data: Vec<u8>,
    checksum: u16,
}

impl Packet {
    pub fn new(header: Header, from: From, to: To, data: Vec<u8>) -> Self {
        Self {
            header,
            from,
            to,
            data,
            checksum: Self::calculate_checksum(data),
        }
    }

    fn calculate_checksum(data: Vec<u8>) -> u16 {
        let mut sum: u16 = 0;
        for byte in data {
            sum.wrapping_add(byte as u16);
        }
        sum
    }
    pub fn to_flits(&self) -> Vec<Flit> {
        // make head flit
        let mut head_flit: Flit = 0;
        unimplemented!();
    }
    pub fn from_flits(flits: Vec<Flit>) -> Result<Packet> {
        let head_flit = flits[0];
        let loader = FlitLoader::new(head_flit)?;
        if loader.flit_type() != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        let unimplemented!();
    }
}
