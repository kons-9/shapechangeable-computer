use crate::serial::Serial;

use super::flit::{Flit, FlitLoader, FlitMaker, FlitType, Id};
use super::header::Header;
use super::protocol::{DefaultProtocol, Protocol};
use super::sender::Sender;
use anyhow::{anyhow, Result};

type FromId = Id;
pub type PacketId = u8;

// broadcast is represented by 0xFFFF
// localnet is only used when making localnet
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ToId {
    Unicast(Id),
    Broadcast,
}

impl ToId {
    fn to_id(&self) -> Id {
        // todo
        match self {
            ToId::Unicast(id) => *id,
            ToId::Broadcast => 0xFFFF,
        }
    }
    fn from_id(id: Id) -> Self {
        // todo
        match id {
            0xFFFF => ToId::Broadcast,
            _ => ToId::Unicast(id),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Packet {
    packet_id: PacketId,
    header: Header,
    from: FromId,
    to: ToId,
    messages: Vec<u8>,
    /// last 2 items represent checksum
    checksum: u16,
    length_of_flit: usize,
}

impl Packet {
    pub fn send_broadcast(&self, serial: &Serial) -> Result<()> {
        let flits = self.to_flits(Option::<&DefaultProtocol>::None);
        for flit in flits {
            flit.send(serial)?;
        }
        Ok(())
    }
    pub fn send(&self, serial: &Serial, protocol: &impl Protocol) -> Result<()> {
        let flits = self.to_flits(Option::Some(protocol));
        for flit in flits {
            flit.send(serial)?;
        }
        Ok(())
    }
    pub fn receive(serial: &Serial) -> Result<Self> {
        let mut flits = Vec::new();
        let flit = Flit::receive(serial)?;
        let (length_of_flit, _, _, _, _) = FlitLoader::get_head_information(flit)?;
        flits.push(flit);

        for _ in 1..length_of_flit {
            flits.push(Flit::receive(serial)?);
        }
        Self::from_flits(flits)
    }
    pub fn new(
        packet_id: PacketId,
        header: Header,
        from: FromId,
        to: ToId,
        mut messages: Vec<u8>,
    ) -> Result<Self> {
        let checksum = Self::calculate_checksum(&messages);
        // checksum is 2 bytes and packet id is 2 bytes

        let length_of_flit = messages.len() + 4;

        for _ in 0..6 {
            messages.push(0);
        }

        Ok(Self {
            packet_id,
            header,
            from,
            to,
            messages,
            checksum,
            length_of_flit,
        })
    }

    fn calculate_checksum(data: &Vec<u8>) -> u16 {
        let mut sum: u16 = 0;
        for byte in data {
            sum = sum.wrapping_add(*byte as u16);
        }
        sum
    }
    pub fn change_from(&mut self, from: FromId) {
        self.from = from;
    }

    pub fn to_flits(&self, routing: Option<&impl Protocol>) -> Vec<Flit> {
        // todo: should remake according to header, because some of them don't need tail_flit owing
        // to size of packet. it is more effecient.
        //
        //
        let id = self.to;
        let next_id = if id == ToId::Broadcast {
            id.to_id()
        } else {
            routing.unwrap().get_next_node(self.from, id.to_id())
        };

        let mut flits = vec![FlitMaker::make_head_flit(
            self.length_of_flit as u8,
            self.header,
            self.from,
            next_id,
            self.packet_id,
        )];

        if self.header.is_only_head() {
            return flits;
        }
        // one message can have 48bit(6byte)
        let mut data = self.make_first_message();

        // add packet id and checksum

        if self.messages.len() == 0 {
            let tail_flit = FlitMaker::make_tail_flit(1, data);
            flits.push(tail_flit);
            return flits;
        }
        let flit_id = 1;
        let body_flit = FlitMaker::make_body_flit(flit_id, data);
        flits.push(body_flit);

        // add message

        for i in 0..((self.length_of_flit - 4) / 6 + 1) {
            for j in 0..6 {
                data[j] = self.messages[i * 6 + j]
            }
            let body_flit = FlitMaker::make_body_flit(flit_id, data);
            flits.push(body_flit);
        }
        FlitMaker::change_flit_type(&mut flits[self.length_of_flit - 1], FlitType::Tail);

        flits
    }
    fn make_first_message(&self) -> [u8; 6] {
        let mut data: [u8; 6] = [0; 6];

        data[0] = self.packet_id;
        let checksum = self.checksum.to_le_bytes();
        data[1] = checksum[0];
        data[2] = checksum[1];
        let ids = self.to.to_id().to_le_bytes();
        data[3] = ids[0];
        data[4] = ids[1];
        return data;
    }
    fn check_checksum(data: &Vec<u8>) -> bool {
        let mut sum: u16 = 0;
        for i in 0..data.len() - 2 {
            sum = sum.wrapping_add(data[i] as u16);
        }
        let checksum = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
        sum == checksum
    }

    pub fn from_flits(flits: Vec<Flit>) -> Result<Packet> {
        let (length_of_flit, header, from, to, packet_id) =
            FlitLoader::get_head_information(flits[0])?;
        let to = ToId::from_id(to);
        let length_of_flit = length_of_flit.into();

        let mut data = Vec::new();

        for i in 1..length_of_flit {
            let (flittype, flit_id, message) = FlitLoader::get_body_or_tail_information(flits[i])?;
            if flit_id as usize != i {
                return Err(anyhow!("The flit id is not correct."));
            }

            if flittype == FlitType::Tail && i != length_of_flit - 1 {
                return Err(anyhow!("The flit is not last but Tail."));
            }

            for j in message {
                data.push(j);
            }
        }
        if Self::check_checksum(&data) {
            Ok(Self::new(packet_id, header, from, to, data)?)
        } else {
            Err(anyhow!("Checksum is not correct"))
        }
    }

    // getter
    pub fn get_packet_id(&self) -> PacketId {
        self.packet_id
    }
    pub fn get_header(&self) -> Header {
        self.header
    }
    pub fn get_from(&self) -> FromId {
        self.from
    }
    pub fn get_to(&self) -> ToId {
        self.to
    }
    pub fn get_messages(self) -> Vec<u8> {
        self.messages
    }
    pub fn get_all(self) -> (PacketId, Header, FromId, Id, Vec<u8>) {
        (
            self.packet_id,
            self.header,
            self.from,
            self.to.to_id(),
            self.messages,
        )
    }
}
/// PacketManager is a struct that makes/loads common packet.
/// It is used for initialization of network.
struct PacketManager;
impl PacketMaker {
    pub fn make_packet(
        packet_id: PacketId,
        header: Header,
        from: FromId,
        to: ToId,
        messages: Vec<u8>,
    ) -> Result<Packet> {
        let packet = Packet::new(packet_id, header, from, to, messages)?;
        Ok(packet)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::network::header::Header;

    #[test]
    fn test() {
        let data: &str = "hello world";
        let packet_data = data.as_bytes().to_vec();
        let packet = Packet::new(0, Header::Data, 0, ToId::Unicast(1), packet_data).unwrap();
        let flits = packet.to_flits(Option::<&DefaultProtocol>::None);
        let trans_packet = Packet::from_flits(flits).unwrap();
        assert_eq!(packet, trans_packet);
        let trans_data = String::from_utf8(trans_packet.messages);
        let trans_data = trans_data.unwrap();
        let trans_data = trans_data.as_str();
        assert_eq!(data, trans_data);
    }
}
