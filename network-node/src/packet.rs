use std::mem::size_of;

use crate::serial::Serial;
use crate::utils::util::{self, is_same_localnet};

use super::flit::{Flit, FlitType, MAX_FLIT_LENGTH};
use super::header::Header;
use crate::utils::type_alias::{Coordinate, CoordinateComponent, Id};
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
    pub fn to_id(&self) -> Id {
        // todo
        match self {
            ToId::Unicast(id) => *id,
            ToId::Broadcast => 0xFFFF,
        }
    }
    pub fn from_id(id: Id) -> Self {
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
    global_from: FromId,
    global_to: ToId,
    messages: Vec<u8>,
    /// last 2 items represent checksum
    checksum: u8,
    length_of_flit: usize,
}

impl Packet {
    // connection
    pub fn send_broadcast(&self, serial: &mut dyn Serial) -> Result<()> {
        let flits = self.to_flits();
        for flit in flits {
            flit.send(serial)?;
        }
        Ok(())
    }
    pub fn send(&self, serial: &mut dyn Serial) -> Result<()> {
        let flits = self.to_flits();
        for flit in flits {
            flit.send(serial)?;
        }
        Ok(())
    }
    pub fn receive(serial: &mut dyn Serial) -> Result<Option<Self>> {
        let mut flits = Vec::new();
        let flit = match Flit::receive(serial)? {
            Some(flit) => flit,
            None => return Ok(None),
        };

        let (length_of_flit, _, _, _, _) = Flit::get_head_information(flit)?;

        flits.push(flit);

        for _ in 1..length_of_flit {
            flits.push(Flit::wait_receive(serial)?);
        }
        Ok(Some(Self::from_flits(flits)?))
    }

    pub fn new(
        packet_id: PacketId,
        header: Header,
        global_from: FromId,
        global_to: ToId,
        from: FromId,
        to: ToId,
        mut messages: Vec<u8>,
    ) -> Self {
        let checksum = Self::calculate_checksum(&messages);
        // checksum is 2 bytes and packet id is 2 bytes

        let length_of_flit = Self::calculate_length_of_flit(header, &messages);

        for _ in 0..6 {
            messages.push(0);
        }

        Self {
            packet_id,
            header,
            from,
            to,
            global_from,
            global_to,
            messages,
            checksum,
            length_of_flit,
        }
    }
    fn calculate_length_of_flit(header: Header, messages: &Vec<u8>) -> usize {
        if header.is_only_head() {
            return 1;
        }
        // headflit(1) + first_messages + messages / 6
        let mut length_of_flit = 2;
        length_of_flit += (messages.len() + 5) / 6;
        length_of_flit
    }

    fn calculate_checksum(data: &Vec<u8>) -> u8 {
        let mut sum: u8 = 0;
        for byte in data {
            sum = sum.wrapping_add(*byte);
        }
        sum
    }
    pub fn change_from_and_to(&mut self, from: FromId, to: ToId) {
        self.from = from;
        self.to = to;
    }

    pub fn to_flits(&self) -> Vec<Flit> {
        // todo: should remake according to header, because some of them don't need tail_flit owing
        // to size of packet. it is more effecient.
        //
        //

        let mut flits = vec![Flit::make_head_flit(
            self.length_of_flit as u8,
            self.header,
            self.from,
            self.to.to_id(),
            self.packet_id,
        )];

        if self.header.is_only_head() {
            return flits;
        }
        // one message can have 48bit(6byte)
        let mut data = self.make_first_message();

        // add packet id and checksum

        if self.messages.len() == 0 {
            let tail_flit = Flit::make_tail_flit(1, data);
            flits.push(tail_flit);
            return flits;
        }
        let mut flit_id = 1;
        let body_flit = Flit::make_body_flit(flit_id, data);
        flits.push(body_flit);

        // add message

        for i in 0..self.length_of_flit - 2 {
            flit_id += 1;
            for j in 0..6 {
                data[j] = self.messages[i * 6 + j]
            }
            let body_flit = Flit::make_body_flit(flit_id, data);
            flits.push(body_flit);
        }
        let last = flits.len() - 1;
        Flit::change_flit_type(&mut flits[last], FlitType::Tail);
        if self.length_of_flit != flits.len() % MAX_FLIT_LENGTH as usize {
            panic!("length of flit is not correct");
        }

        flits
    }
    fn make_first_message(&self) -> [u8; 6] {
        let mut data: [u8; 6] = [0; 6];

        data[0] = self.packet_id;
        data[1] = self.checksum;
        let ids = self.global_to.to_id().to_be_bytes();
        data[2] = ids[0];
        data[3] = ids[1];
        let ids = self.global_from.to_be_bytes();
        data[4] = ids[0];
        data[5] = ids[1];
        return data;
    }
    fn load_first_message(flit: Flit) -> (PacketId, u8, Id, Id) {
        let data = flit.to_be_bytes();
        let packet_id = data[0];
        let checksum = data[1];
        let to = Id::from_be_bytes([data[2], data[3]]);
        let from = Id::from_be_bytes([data[4], data[5]]);
        (packet_id, checksum, from, to)
    }

    pub fn from_flits(flits: Vec<Flit>) -> Result<Packet> {
        if flits.len() == 0 {
            return Err(anyhow!("The length of flits is zero."));
        }
        let (length_of_flit, header, from, to, packet_id) = Flit::get_head_information(flits[0])?;
        let to = ToId::from_id(to);
        let length_of_flit = length_of_flit.into();

        if header.is_only_head() {
            return Ok(Self::new(packet_id, header, from, to, from, to, Vec::new()));
        }

        // general packet has at least 2 flits
        if flits.len() < 2 {
            return Err(anyhow!("The length of flits is not enough."));
        }

        let (packet_id, checksum, global_source, global_destination) =
            Self::load_first_message(flits[1]);

        let mut data = Vec::new();

        for i in 2..length_of_flit {
            let (flittype, flit_id, message) = Flit::get_body_or_tail_information(flits[i])?;
            if flit_id as usize != i {
                #[cfg(test)]
                assert_eq!(flit_id as usize, i, "The flit id is not correct.");
                return Err(anyhow!("The flit id is not correct."));
            }

            if flittype == FlitType::Tail && i != length_of_flit - 1 {
                return Err(anyhow!("The flit is not last but Tail."));
            }

            for j in message {
                data.push(j);
            }
        }
        if Self::check_checksum(&data, checksum) {
            Ok(Self::new(
                packet_id,
                header,
                global_source,
                ToId::from_id(global_destination),
                from,
                to,
                data,
            ))
        } else {
            Err(anyhow!(
                "Checksum is not correct: {:x}, {:x}",
                checksum,
                Self::calculate_checksum(&data)
            ))
        }
    }
    fn check_checksum(data: &Vec<u8>, checksum: u8) -> bool {
        let sum = Self::calculate_checksum(data);
        sum == checksum
    }

    // ///////////////////////////////
    // Packet Maker
    // ///////////////////////////////
    // these function is for making/loading iregular packet.
    // It is mainly used for initialization of network.
    //
    pub fn make_check_connection_packet(source: Id) -> Packet {
        let packet = Self::new(
            0,
            Header::HCheckConnection,
            source,
            ToId::Broadcast,
            source,
            ToId::Broadcast,
            Vec::new(),
        );
        packet
    }
    pub fn make_reply_for_request_confirmed_coordinate_packet(
        source: Id,
        destination: Id,
        coordinate: &Vec<(Id, Coordinate)>,
        this_coordinate: Option<Coordinate>,
    ) -> Result<Packet> {
        // if the source node is confirmed, coordinate is only one, which is the coordiate of source node.
        let header = Header::ConfirmCoordinate;
        let packet_id = 0;
        let from = source;
        let to = ToId::Broadcast;
        let global_from = source;
        let global_to = ToId::Broadcast;
        let mut messages = Vec::new();

        if !is_same_localnet(source, destination) {
            return Self::make_reply_for_request_confirmed_coordinate_packet_in_different_localnet(
                source,
                this_coordinate,
            );
        }
        let is_confirmed = this_coordinate.is_some();
        if is_confirmed {
            messages.push(0b11111111);
        } else {
            messages.push(0);
        }
        for (_, i) in coordinate {
            let id = source.to_be_bytes();
            messages.push(id[0]);
            messages.push(id[1]);
            let x = i.0.to_be_bytes();
            messages.push(x[0]);
            messages.push(x[1]);
            let y = i.1.to_be_bytes();
            messages.push(y[0]);
            messages.push(y[1]);
        }
        Ok(Self::new(
            packet_id,
            header,
            from,
            to,
            global_from,
            global_to,
            messages,
        ))
    }
    fn make_reply_for_request_confirmed_coordinate_packet_in_different_localnet(
        source: Id,
        this_coordinate: Option<Coordinate>,
    ) -> Result<Packet> {
        if this_coordinate.is_none() {
            return Err(anyhow!("The coordinate is not correct."));
        }
        let this_coordinate = this_coordinate.unwrap();
        let mut messages = Vec::new();
        messages.push(0b11111111);
        let id = source.to_be_bytes();
        messages.push(id[0]);
        messages.push(id[1]);
        let x = this_coordinate.0.to_be_bytes();
        messages.push(x[0]);
        messages.push(x[1]);
        let y = this_coordinate.1.to_be_bytes();
        messages.push(y[0]);
        messages.push(y[1]);

        let header = Header::ConfirmCoordinate;
        let packet_id = 0;
        let from = source;
        let to = ToId::Broadcast;
        let global_from = source;
        let global_to = ToId::Broadcast;

        Ok(Self::new(
            packet_id,
            header,
            from,
            to,
            global_from,
            global_to,
            messages,
        ))
    }
    /// make broudcast packet
    pub fn make_request_confirmed_coordinate_packet(source: Id) -> Packet {
        // only head flit
        let header = Header::HRequestConfirmedCoordinate;
        let packet_id = 0;
        let from = source;
        let to = ToId::Broadcast;
        let global_from = source;
        let global_to = ToId::Broadcast;
        let messages = Vec::new();
        Self::new(
            packet_id,
            header,
            from,
            to,
            global_from,
            global_to,
            messages,
        )
    }

    // ///////////////////////////////
    // Packet Loader
    // ///////////////////////////////
    //
    pub fn load_confirmed_coordinate_packet(&self, source_id: Id) -> Result<Vec<(Id, Coordinate)>> {
        // load coordinate of node that is in the same localnet
        // data is like this [ is_confirmed(8) | id(16) | x(16) | y(16) | id(16)...]

        let messages = self.get_ref_messages();
        // id size + x size + y size
        const UNIT_BYTE: usize =
            (size_of::<CoordinateComponent>() * 2 + size_of::<Id>()) / size_of::<u8>();
        // messages length is 1(is_confirmed section) + 6 * n
        if (messages.len() - 1) % UNIT_BYTE != 0 {
            panic!("length of message is not correct");
        }
        let is_confirmed = messages[0] != 0;

        // confirmed and source_id is in the same localnet
        if is_confirmed && !util::is_same_localnet(self.get_global_from(), source_id) {
            return Ok(Vec::new());
        }

        let mut coordinates = Vec::new();
        for i in (1..messages.len()).step_by(6) {
            let id = Id::from_be_bytes([messages[i], messages[i + 1]]);
            let x = CoordinateComponent::from_be_bytes([messages[i + 2], messages[i + 3]]);
            let y = CoordinateComponent::from_be_bytes([messages[i + 4], messages[i + 5]]);
            coordinates.push((id, (x, y)));
        }
        if is_confirmed && coordinates.len() != 1 {
            return Err(anyhow!(
                "This node is confirmed but the number of coordinate is not 1."
            ));
        }
        return Ok(coordinates);
    }
    // ///////////////////////////////
    // Packet Utils
    // ///////////////////////////////
    //

    // ///////////////////////////////
    // getter
    // ///////////////////////////////
    pub fn get_packet_id(&self) -> PacketId {
        self.packet_id
    }
    pub fn get_header(&self) -> Header {
        self.header
    }
    pub fn get_global_from(&self) -> Id {
        self.global_from
    }
    pub fn get_global_to(&self) -> ToId {
        self.global_to
    }
    pub fn get_from(&self) -> FromId {
        self.from
    }
    pub fn get_to(&self) -> ToId {
        self.to
    }
    pub fn get_ref_messages(&self) -> &Vec<u8> {
        &self.messages
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

#[cfg(test)]
mod test {
    #![allow(unused_imports)]
    use super::*;
    use crate::header::Header;

    #[test]
    fn test_broadcast() {
        let data: &str = "hello world";
        let packet_data = data.as_bytes().to_vec();
        let packet = Packet::new(
            0,
            Header::Data,
            0,
            ToId::Broadcast,
            0,
            ToId::Broadcast,
            packet_data,
        );
        let flits = packet.to_flits();
        let trans_packet = Packet::from_flits(flits).unwrap();
        assert_eq!(packet, trans_packet);
        let trans_data = String::from_utf8(trans_packet.messages);
        let trans_data = trans_data.unwrap();
        let trans_data = trans_data.as_str();
        assert_eq!(data, trans_data);
    }
}
