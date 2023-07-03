use super::flit::{Coordinate, Flit, FlitLoader, FlitMethod, FlitType, Id};
use super::header::Header;
use anyhow::{anyhow, Result};

type From = Id;

// broadcast is represented by 0xFFFF
// localnet is only used when making localnet
enum To {
    Localnet(Id),
    Node(Id),
    Broadcast,
}
impl To {
    fn to_id(&self) -> Id {
        // todo
        match self {
            To::Localnet(id) => *id,
            To::Node(id) => *id,
            To::Broadcast => 0xFFFF,
        }
    }
    fn from_id(id: Id) -> Self {
        // todo
        match id {
            0xFFFF => To::Broadcast,
            _ => To::Node(id),
        }
    }
}

struct Packet {
    header: Header,
    from: From,
    to: To,
    /// last 2 items represent checksum
    data: Vec<u8>,
    length_of_flit: usize,
}

impl Packet {
    pub fn new(header: Header, from: From, to: To, mut data: Vec<u8>) -> Self {
        let checksum = Self::calculate_checksum(&data);
        data.push((checksum >> 8) as u8);
        data.push((checksum & 0xFF) as u8);
        let length_of_flit = data.len() + 1;

        for _ in 0..3 {
            data.push(0);
        }

        Self {
            header,
            from,
            to,
            data,
            length_of_flit,
        }
    }

    fn calculate_checksum(data: &Vec<u8>) -> u16 {
        let mut sum: u16 = 0;
        for byte in data {
            sum.wrapping_add(*byte as u16);
        }
        sum
    }
    pub fn to_flits(&self) -> Vec<Flit> {
        // todo: should remake according to header, because some of them don't need tail_flit owing
        // to size of packet. it is more effecient.
        //
        let mut flits = vec![Flit::make_head_flit(
            self.header,
            self.from,
            self.to.to_id(),
            self.length_of_flit as u8,
        )];

        if self.header.is_only_head() {
            return flits;
        }

        for i in 0..(self.length_of_flit / 3 + 1) {
            let mut data: u64 = 0;
            for j in 0..3 {
                data |= (self.data[i * 3 + j] as u64) << (8 * j);
            }
            let body_flit = Flit::make_body_flit(data);
            flits.push(body_flit);
        }
        flits[self.length_of_flit - 1].change_flit_type(FlitType::Tail);
        flits
    }

    pub fn from_flits(flits: Vec<Flit>) -> Result<Packet> {
        let head_flit = flits[0];
        let loader = FlitLoader::new(head_flit)?;
        if loader.flit_type() != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        let header = loader.get_header()?;
        let from = loader.get_source_id()?;
        let to = To::from_id(loader.get_destination_id()?);
        let length_of_flit = loader.get_length()?.into();
        let mut data = Vec::new();

        for i in 1..length_of_flit {
            let loader = FlitLoader::new(flits[i])?;
            if loader.flit_type() != FlitType::Body {
                return Err(anyhow!("This flit is not Body"));
            }
            let data = loader.get_data()?;
            for j in 0..3 {
                data.push((data >> (8 * j)) as u8);
            }
        }

        return Ok(packet);
    }
}
