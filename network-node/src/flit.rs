use super::header::Header;
use super::packet::PacketId;
use super::serial::SerialTrait;
use crate::utils::type_alias::Id;
use anyhow::anyhow;
use anyhow::Result;
use log::info;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::ops;

/// Flit consists of 64 bits.
/// HeadFlit : [ FlitType(2) | LengthOfFlit(6) | Header(8) | SourceId(16) | DestinationId(16) | PacketId(8) | Checksum(8) ]
/// Body and TailFlit : [ FlitType(2) | FlitId(6) | Message(48) | Checksum(8)]
/// NopeFlit : [ FlitType(2) | z(undefined)(62) ]
#[derive(Debug, Clone, Copy)]
pub struct Flit(u64);

// 6bits
type FlitId = u8;

#[derive(TryFromPrimitive, PartialEq, Debug)]
#[repr(u8)]
pub enum FlitType {
    // 2bits
    Nope = 0, // 00
    Head = 1, // 01
    Body = 2, // 10
    Tail = 3, // 11
}

const TIMEOUT_MILLIS: u64 = 100000;
const DELAY_MILLIS: u64 = 1000;
const MAX_LOOPS: u64 = TIMEOUT_MILLIS / DELAY_MILLIS;
pub const MAX_FLIT_LENGTH: FlitId = 64;

impl Flit {
    // ////////////////////////////////
    // Flit Sender
    // ////////////////////////////////

    pub fn send_broadcast(&self, serial: &mut impl SerialTrait) -> Result<()> {
        // we don't need to receive ack
        serial.send(&self.to_be_bytes())?;
        Ok(())
    }
    pub fn send(&self, serial: &mut dyn SerialTrait, is_ack: bool) -> Result<()> {
        if !is_ack {
            info!("don't require ack flit");
            serial.send(&self.to_be_bytes())?;
            return Ok(());
        }

        info!("require ack flit");
        // require ack flit
        loop {
            serial.send(&self.to_be_bytes())?;

            // receive ack
            let mut loop_cnt = MAX_LOOPS;
            loop {
                // 10ms delay
                std::thread::sleep(std::time::Duration::from_millis(DELAY_MILLIS));
                if loop_cnt > 100 {
                    return Err(anyhow!("ack timeout"));
                }
                loop_cnt += 1;

                let receive = serial.receive()?;
                if let Option::<[u8; 8]>::None = receive {
                    continue;
                }

                let ack_flit: Flit = Self::from_be_bytes(receive.unwrap());
                if Flit::check_ack_flit(ack_flit, *self)? {
                    return Ok(());
                }
            }
        }
    }

    pub fn wait_receive(serial: &mut dyn SerialTrait) -> Result<Self> {
        let mut loop_cnt = 0;
        let flit: Flit;
        loop {
            if loop_cnt > 100 {
                return Err(anyhow!("ack timeout"));
            }
            // 10ms delay
            std::thread::sleep(std::time::Duration::from_millis(10));
            loop_cnt += 1;
            let receive = serial.receive()?;
            if let Option::<[u8; 8]>::None = receive {
                continue;
            }
            flit = Flit::from_be_bytes(receive.unwrap());
            break;
        }
        flit.process_ack(serial)?;
        return Ok(flit);
    }

    pub fn receive(serial: &mut dyn SerialTrait) -> Result<Option<Self>> {
        // receive
        let receive = serial.receive()?;
        if let Option::<[u8; 8]>::None = receive {
            return Ok(None);
        }
        let flit = Flit::from_be_bytes(receive.unwrap());
        flit.process_ack(serial)?;

        return Ok(Some(flit));
    }

    pub fn process_ack(&self, serial: &mut dyn SerialTrait) -> Result<()> {
        let (flittype, _) = Flit::get_flit_type_and_length(self)?;
        if flittype != FlitType::Head {
            return Ok(());
        }

        if self.get_header()?.is_require_ack() {
            let ack_flit = Flit::make_ack_flit(&self)?;
            serial.send(&ack_flit.to_be_bytes())?;
        }
        Ok(())
    }

    // ////////////////////////////////
    // Flit Maker
    // ////////////////////////////////

    fn set_2_6bits(val2bit: u8, val6bit: u8) -> u8 {
        let mut data: u8 = 0;
        data |= (val2bit << 6) & 0b11000000;
        data |= val6bit & 0b00111111;
        data
    }
    pub fn make_ack_flit(flit: &Flit) -> Result<Flit> {
        info!("make ack flit from {:?}", flit);
        let (_, _, source_id, destination_id, packet_id) = Flit::get_head_information(flit)?;

        Ok(Self::make_head_flit(
            0,
            Header::HAck,
            source_id,
            destination_id,
            packet_id,
        ))
    }

    pub fn make_head_flit(
        len_of_flit: FlitId,
        header: Header,
        source_id: Id,
        destination_id: Id,
        packet_id: u8,
    ) -> Flit {
        let mut flitbyte = [0; 8];

        let flittype = FlitType::Head as u8;
        let len_of_flit = len_of_flit as u8;
        flitbyte[0] = Self::set_2_6bits(flittype, len_of_flit);
        let header = header as u8;
        flitbyte[1] = header;
        let source_id = source_id.to_be_bytes();
        flitbyte[2] = source_id[0];
        flitbyte[3] = source_id[1];
        let destination_id = destination_id.to_be_bytes();
        flitbyte[4] = destination_id[0];
        flitbyte[5] = destination_id[1];
        let packet_id = packet_id.to_be_bytes();
        flitbyte[6] = packet_id[0];

        let checksum = Self::calculate_checksum(&flitbyte);
        flitbyte[7] = checksum;

        Flit::from_be_bytes(flitbyte)
    }
    fn make_body_or_tail_flit(flittype: FlitType, flit_id: FlitId, message: [u8; 6]) -> Flit {
        let mut flitbyte = [0; 8];
        let flittype = flittype as u8;
        flitbyte[0] = Self::set_2_6bits(flittype, flit_id);
        flitbyte[1] = message[0];
        flitbyte[2] = message[1];
        flitbyte[3] = message[2];
        flitbyte[4] = message[3];
        flitbyte[5] = message[4];
        flitbyte[6] = message[5];
        let checksum = Self::calculate_checksum(&flitbyte);
        flitbyte[7] = checksum;

        Flit::from_be_bytes(flitbyte)
    }
    pub fn make_body_flit(flit_id: FlitId, message: [u8; 6]) -> Flit {
        let flittype = FlitType::Body;
        Self::make_body_or_tail_flit(flittype, flit_id, message)
    }
    pub fn make_tail_flit(flit_id: FlitId, message: [u8; 6]) -> Flit {
        let flittype = FlitType::Tail;
        Self::make_body_or_tail_flit(flittype, flit_id, message)
    }
    #[allow(dead_code)]
    pub fn make_nope_flit() -> Flit {
        Flit(0)
    }
    fn clear_flit_type(flit: &mut Flit) {
        *flit &= !(0b11 << 62);
    }
    pub fn change_flit_type(flit: &mut Flit, flit_type: FlitType) {
        Self::clear_flit_type(flit);
        Self::set_flit_type(flit, flit_type);
        flit.0 &= !(0b11111111);
        let sum = Self::calculate_checksum(&flit.to_be_bytes());
        flit.0 |= sum as u64;
    }
    fn set_flit_type(flit: &mut Flit, flit_type: FlitType) {
        *flit |= (flit_type as u64) << 62;
    }

    fn calculate_checksum(flitbyte: &[u8; 8]) -> u8 {
        let mut sum: u8 = 0;
        for byte in flitbyte {
            sum = sum.wrapping_add(*byte);
        }
        sum
    }

    // ////////////////////////////////
    // Flit Loader
    // ////////////////////////////////

    // utils
    #[allow(dead_code)]
    fn get_u16_from_u64(val: u64, start: u8) -> u16 {
        TryFrom::try_from((val >> start) & u16::MAX as u64).unwrap()
    }
    #[allow(dead_code)]
    fn get_u8_from_u64(val: u64, start: u8) -> u8 {
        TryFrom::try_from((val >> start) & u8::MAX as u64).unwrap()
    }
    fn get_2bits_from_u64(val: u64, start: u8) -> u8 {
        TryFrom::try_from((val >> start) & 0b11 as u64).unwrap()
    }
    fn get_6bits_from_u64(val: u64, start: u8) -> u8 {
        TryFrom::try_from((val >> start) & 0b111111 as u64).unwrap()
    }

    // for all
    /// if type is not head, return (flittype, flit_id)
    fn get_flit_type_and_length(flit: &Flit) -> Result<(FlitType, u8)> {
        // the toppest 2 bits
        // note: it is big endian
        let flit_type = Flit::get_2bits_from_u64(flit.0, 62);
        let flit_length = Flit::get_6bits_from_u64(flit.0, 56);
        Ok((FlitType::try_from(flit_type)?, flit_length))
    }
    pub(crate) fn check_ack_flit(ack_flit: Flit, original_flit: Flit) -> Result<bool> {
        // pull from get_head_information
        let (_, header, _source_id, destination_id, packet_id) =
            Flit::get_head_information(&ack_flit)?;
        let (_, _, original_source_id, _original_destination_id, original_packet_id) =
            Flit::get_head_information(&original_flit)?;
        if header != Header::HAck {
            return Ok(false);
        }
        if original_packet_id != packet_id {
            return Ok(false);
        }
        if original_source_id != destination_id {
            return Ok(false);
        }

        Ok(true)
    }

    /// return (length_of_flit, header, source_id, destination_id, packet_id)
    pub fn get_head_information(flit: &Flit) -> Result<(u8, Header, Id, Id, PacketId)> {
        let bytes: [u8; 8] = flit.to_be_bytes();
        let (flit_type, length_of_flit) = Flit::get_flit_type_and_length(flit)?;
        if flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }

        let header = Header::try_from(bytes[1])?;
        let source_id = u16::from_be_bytes([bytes[2], bytes[3]]);
        let destination_id = u16::from_be_bytes([bytes[4], bytes[5]]);
        let packet_id = bytes[6];
        let checksum = bytes[7];
        let mut sum: u8 = 0;
        for i in 0..=6 {
            sum = sum.wrapping_add(bytes[i]);
        }
        if sum == checksum {
            Ok((length_of_flit, header, source_id, destination_id, packet_id))
        } else {
            #[cfg(test)]
            assert_eq!(
                sum, checksum,
                "Checksum is not correct in flit.rs(get_head_information): bytes: {:?}",
                bytes
            );
            Err(anyhow!("Checksum is not correct(get_head_information)"))
        }
    }
    pub fn get_body_or_tail_information(flit: &Flit) -> Result<(FlitType, u8, [u8; 6])> {
        let bytes: [u8; 8] = flit.to_be_bytes();
        let (flit_type, flit_id) = Flit::get_flit_type_and_length(flit)?;
        if flit_type == FlitType::Head {
            #[cfg(test)]
            assert_eq!(flit_type, FlitType::Head, "This flit is Head");
            return Err(anyhow!("This flit is Head"));
        }

        let message = [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6]];
        let checksum = bytes[7];
        // self.check_checksum_for_body_and_tail()?;
        let mut sum: u8 = 0;
        for i in 0..=6 {
            sum = sum.wrapping_add(bytes[i]);
        }
        if checksum == sum {
            Ok((flit_type, flit_id, message))
        } else {
            #[cfg(test)]
            assert_eq!(
                checksum, sum,
                "Checksum is not correct in flit.rs(get_body_or_tail_information): bytes: {:?}, flit: {:?}",
                bytes,flit
            );
            Err(anyhow!(
                "Checksum is not correct (get_body_or_tail_information)"
            ))
        }
    }
    pub fn get_header(&self) -> Result<Header> {
        let header = Self::get_u8_from_u64(self.0, 48);
        Ok(Header::try_from(header)?)
    }

    // ////////////////////////////////
    // Utils
    // ////////////////////////////////
    pub fn to_be_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn from_be_bytes(bytes: [u8; 8]) -> Flit {
        Flit(u64::from_be_bytes(bytes))
    }
}

impl ops::BitOrAssign<u64> for Flit {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs;
    }
}
impl ops::BitAndAssign<u64> for Flit {
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 &= rhs;
    }
}
impl ops::BitAnd<u64> for Flit {
    type Output = u64;
    fn bitand(self, rhs: u64) -> Self::Output {
        self.0 & rhs
    }
}
impl ops::BitOr<u64> for Flit {
    type Output = u64;
    fn bitor(self, rhs: u64) -> Self::Output {
        self.0 | rhs
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use crate::serial::test::TestSerial;

    #[test]
    fn test_set_2_6bits() {
        assert_eq!(Flit::set_2_6bits(0b01, 0b111111), 0b01111111);
        assert_eq!(Flit::set_2_6bits(0b10, 0b111111), 0b10111111);
        assert_eq!(Flit::set_2_6bits(0b11, 0b000000), 0b11000000);
        assert_eq!(Flit::set_2_6bits(FlitType::Head as u8, 0), 0b01000000);
    }

    #[test]
    fn test_simple_head_flit() {
        // make head flit
        let flit = Flit::make_head_flit(0, Header::Data, 0, 1, 0);
        assert_eq!(Header::Data as u8, 0b00000000);

        let (flit_type, length_of_flit) = Flit::get_flit_type_and_length(&flit).unwrap();
        assert_eq!(
            flit_type,
            FlitType::Head,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(
            length_of_flit, 0,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );

        let (length_of_flit, header, source_id, destination_id, packet_id) =
            Flit::get_head_information(&flit).unwrap();
        assert_eq!(length_of_flit, 0);
        assert_eq!(header, Header::Data);
        assert_eq!(source_id, 0);
        assert_eq!(destination_id, 1);
        assert_eq!(packet_id, 0);
    }
    #[test]
    fn test_simple_body_or_tail_information() {
        // body flit
        let flit = Flit::make_body_flit(0, [0; 6]);
        let (flit_type, flit_id, message) = Flit::get_body_or_tail_information(&flit).unwrap();
        assert_eq!(
            flit_type,
            FlitType::Body,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(flit_id, 0, "fail get_flit_type: flit {:064b}", flit.0);
        assert_eq!(message, [0; 6], "fail get_flit_type: flit {:064b}", flit.0);
        let flit = Flit::make_body_flit(0b111111, [0, 1, 2, 3, 4, 5]);
        let (flit_type, flit_id, message) = Flit::get_body_or_tail_information(&flit).unwrap();
        assert_eq!(
            flit_type,
            FlitType::Body,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(
            flit_id, 0b111111,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(
            message,
            [0, 1, 2, 3, 4, 5],
            "fail get_flit_type: flit {:064b}",
            flit.0
        );

        // tail flit
        let flit = Flit::make_tail_flit(0, [0; 6]);
        let (flit_type, flit_id, message) = Flit::get_body_or_tail_information(&flit).unwrap();
        assert_eq!(
            flit_type,
            FlitType::Tail,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(flit_id, 0, "fail get_flit_type: flit {:064b}", flit.0);
        assert_eq!(message, [0; 6], "fail get_flit_type: flit {:064b}", flit.0);

        let flit = Flit::make_tail_flit(0b111111, [0, 1, 2, 3, 4, 5]);
        let (flit_type, flit_id, message) = Flit::get_body_or_tail_information(&flit).unwrap();
        assert_eq!(
            flit_type,
            FlitType::Tail,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(
            flit_id, 0b111111,
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
        assert_eq!(
            message,
            [0, 1, 2, 3, 4, 5],
            "fail get_flit_type: flit {:064b}",
            flit.0
        );
    }

    #[test]
    fn test_get_header() {
        let flit = Flit::make_head_flit(0, Header::Data, 0, 1, 0);
        assert_eq!(flit.get_header().unwrap(), Header::Data);
        let flit = Flit::make_head_flit(0, Header::HCheckConnection, 0, 1, 0);
        assert_eq!(flit.get_header().unwrap(), Header::HCheckConnection);
    }

    #[test]
    fn test_for_flit_with_display() {
        let ip_address = 0;
        let header = Header::HCheckConnection;
        let flit = Flit::make_head_flit(0, header, ip_address, 0xFF, 0);
        let (length_of_flit, header, source_id, destination_id, packet_id) =
            Flit::get_head_information(&flit).unwrap();
        assert_eq!(length_of_flit, 0);
        assert_eq!(header, Header::HCheckConnection);
        assert_eq!(source_id, 0);
        assert_eq!(destination_id, 0xFF);
        assert_eq!(packet_id, 0);

        let mut serial = TestSerial::new();
        flit.send(&mut serial, header.is_require_ack()).unwrap();
        println!("flit in serial: {:?}", serial.data[0]);

        match Flit::receive(&mut serial) {
            Ok(data) => {
                if let Some(data) = data {
                    if let Ok((len, head, src, dst, pid)) = Flit::get_head_information(&data) {
                        assert_eq!(len, 0);
                        assert_eq!(head, Header::HCheckConnection);
                        assert_eq!(src, 0);
                        assert_eq!(dst, 0xFF);
                        assert_eq!(pid, 0);
                    }
                } else {
                    panic!("data is None");
                }
            }
            Err(e) => {
                panic!("error: {}", e);
            }
        }
    }
    #[test]
    fn test_get_flit_type_and_length() {
        let flit = Flit::make_head_flit(0, Header::Data, 0, 1, 0);

        let (flit_type, length_of_flit) = Flit::get_flit_type_and_length(&flit).unwrap();
        assert_eq!(flit_type, FlitType::Head);
        assert_eq!(length_of_flit, 0);

        let flit = Flit::make_body_flit(0, [0; 6]);
        let (flit_type, length_of_flit) = Flit::get_flit_type_and_length(&flit).unwrap();
        assert_eq!(flit_type, FlitType::Body);
        assert_eq!(length_of_flit, 0);

        let flit = Flit::make_tail_flit(0, [0; 6]);
        let (flit_type, length_of_flit) = Flit::get_flit_type_and_length(&flit).unwrap();
        assert_eq!(flit_type, FlitType::Tail);
        assert_eq!(length_of_flit, 0);
    }

    #[test]
    fn test_loader_utils() {
        assert_eq!(
            Flit::get_2bits_from_u64(0b001111, 3),
            0b01,
            "fail get_2bits"
        );
        assert_eq!(
            Flit::get_2bits_from_u64(0b001111, 0),
            0b11,
            "fail get_2bits"
        );
        assert_eq!(
            Flit::get_6bits_from_u64(0b111110011, 1),
            0b111001,
            "fail get_6bits"
        );
        assert_eq!(
            Flit::get_u8_from_u64(0b010100101110011111, 3),
            0b01110011,
            "fail get_u8"
        );
        assert_eq!(
            Flit::get_u16_from_u64(0b11111101111110110111, 2),
            0b1111011111101101,
            "fail get_u16"
        );
    }
}
