use super::header::Header;
use anyhow::anyhow;
use anyhow::Result;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

/// Flit consists of 64 bits.
/// HeadFlit : [ FlitType(8) | Header(8) | SourceId(16) | DestinationId(16) | NumOfFlit(8) | Checksum(8) ]
/// Body and TailFlit : [ FlitType(8) | Message(48) | Checksum(8)]
/// NopeFlit : [ FlitType(8) | z(undefined)(56) ]
pub type Flit = u64;
// pub struct Flit(u64);
pub type Id = u16;
pub type Coordinate = (u16, u16);

#[derive(TryFromPrimitive, PartialEq, Debug)]
#[repr(u8)]
pub enum FlitType {
    Head,
    Body,
    Tail,
    Nope,
}
pub trait FlitMethod {
    fn make_head_flit(header: Header, sourceid: Id, destinationid: Id, lenofflit: u8) -> Self;
    fn make_body_flit(message: u64) -> Self;
    fn make_tail_flit(message: u64) -> Self;
    fn change_flit_type(&mut self, flit_type: FlitType);
    // fn get_flit_type(&self) -> FlitType;
    // fn get_header(&self) -> Header;
    // fn get_source_id(&self) -> Id;
    // fn get_destination_id(&self) -> Id;
    // fn get_message(&self) -> u64;
    // fn get_checksum(&self) -> u8;
    //
    // these function should not be used outside of this module
    fn clear_flit_type(&mut self);
    fn set_length_of_flit(&mut self, numofflit: u8);
    fn set_flit_type(&mut self, flit_type: FlitType);
    fn set_header(&mut self, header: Header);
    fn set_source_id(&mut self, source_id: Id);
    fn set_destination_id(&mut self, destination_id: Id);
    fn set_message(&mut self, message: u64);
    fn set_checksum(&mut self);
    fn calculate_checksum(&self) -> u8;
}

impl FlitMethod for Flit {
    fn make_head_flit(header: Header, sourceid: Id, destinationid: Id, lenofflit: u8) -> Self {
        let mut flit: Flit = 0;
        flit.set_flit_type(FlitType::Head);
        flit.set_header(header);
        flit.set_source_id(sourceid);
        flit.set_destination_id(destinationid);
        flit.set_length_of_flit(lenofflit);
        flit.set_checksum();
        flit
    }
    fn make_body_flit(message: u64) -> Self {
        let mut flit: Flit = 0;
        flit.set_flit_type(FlitType::Body);
        flit.set_message(message);
        flit.set_checksum();
        flit
    }
    fn make_tail_flit(message: u64) -> Self {
        let mut flit: Flit = 0;
        flit.set_flit_type(FlitType::Body);
        flit.set_message(message);
        flit.set_checksum();
        flit
    }
    fn clear_flit_type(&mut self) {
        *self &= 0x0000_FFFF_FFFF_FFFF;
    }
    fn change_flit_type(&mut self, flit_type: FlitType) {
        self.clear_flit_type();
        self.set_flit_type(flit_type);
    }

    fn set_flit_type(&mut self, flit_type: FlitType) {
        let u8_value = flit_type as u8;
        *self |= (u8_value as u64) << 56;
    }
    fn set_header(&mut self, header: Header) {
        let u8_value = header as u8;
        *self |= (u8_value as u64) << 48;
    }
    fn set_source_id(&mut self, sourceid: Id) {
        *self |= (sourceid as u64) << 32;
    }
    fn set_destination_id(&mut self, destinationid: Id) {
        *self |= (destinationid as u64) << 16;
    }
    fn set_length_of_flit(&mut self, lenofflit: u8) {
        *self |= (lenofflit as u64) << 8;
    }
    fn set_checksum(&mut self) {
        let checksum = self.calculate_checksum();
        *self |= checksum as u64;
    }
    fn set_message(&mut self, message: u64) {
        // message is like 0x0000_xxxx_xxxx_xxxx
        *self |= message << 8;
    }
    fn calculate_checksum(&self) -> u8 {
        let mut sum: u8 = 0;
        let mut value = *self;
        for _ in 0..4 {
            sum = sum.wrapping_add((value) as u8);
            value >>= 8;
        }
        sum
    }
}

/// FlitLoader is a struct for loading flit.
pub(crate) struct FlitLoader {
    flit: Flit,
    flit_type: FlitType,
}

impl FlitLoader {
    pub(crate) fn new(flit: Flit) -> Result<Self> {
        Ok(FlitLoader {
            flit,
            flit_type: FlitLoader::get_flit_type(flit)?,
        })
    }
    pub fn flit_type(&self) -> FlitType {
        self.flit_type
    }
    pub fn flit(&self) -> Flit {
        self.flit
    }

    // utils
    fn get_u16_from_u64(val: u64, start: u8) -> u16 {
        TryFrom::try_from((val >> start) & u16::MAX as u64).unwrap()
    }
    fn get_u8_from_u64(val: u64, start: u8) -> u8 {
        TryFrom::try_from((val >> start) & u8::MAX as u64).unwrap()
    }

    // for all
    fn get_flit_type(flit: Flit) -> Result<FlitType> {
        // the toppest 8 bits
        let u8_value = (flit as u64 >> 56) as u8;
        Ok(FlitType::try_from(u8_value)?)
    }

    pub(crate) fn get_head_information(&self) -> Result<(Header, Id, Id, u8)> {
        let bytes: [u8; 8] = self.flit.to_le_bytes();
        let flit_type = FlitType::try_from(bytes[0]).unwrap();
        if flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        let header = Header::try_from(bytes[1]).unwrap();
        let source_id = u16::from_le_bytes([bytes[2], bytes[3]]);
        let destination_id = u16::from_le_bytes([bytes[4], bytes[5]]);
        let length_of_flit = bytes[6];
        let checksum = bytes[7];
        let mut sum: u8 = 0;
        for i in 0..6 {
            sum = sum.wrapping_add(bytes[i]);
        }
        if sum == checksum {
            Ok((header, source_id, destination_id, length_of_flit))
        } else {
            Err(anyhow!("Checksum is not correct"))
        }
    }
    pub(crate) fn get_body_information(&self) -> Result<u64> {
        let bytes: [u8; 8] = self.flit.to_le_bytes();
        let flit_type = FlitType::try_from(bytes[0]).unwrap();
        if flit_type != FlitType::Body {
            return Err(anyhow!("This flit is not Body"));
        }
        let message = u64::from_le_bytes([
            0, 0, bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
        ]);
        let checksum = bytes[7];
        // self.check_checksum_for_body_and_tail()?;
        let mut sum: u8 = 0;
        for i in 0..6 {
            sum = sum.wrapping_add(bytes[i]);
        }
        Ok(message)
    }

    // for head
    pub(crate) fn get_source_id(&self) -> Result<u16> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 16 bits
        let u16_value = Self::get_u16_from_u64(self.flit, 40);
        Ok(u16_value)
    }

    pub(crate) fn get_destination_id(&self) -> Result<u16> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 16 bits
        let u16_value = Self::get_u16_from_u64(self.flit, 24);
        Ok(u16_value)
    }

    pub(crate) fn get_header(&self) -> Result<Header> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 8 bits
        let u8_value = Self::get_u8_from_u64(self.flit, 16);
        Ok(Header::try_from(u8_value)?)
    }

    pub(crate) fn get_length(&self) -> Result<u8> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 8 bits
        let u8_value = Self::get_u8_from_u64(self.flit, 8);
        Ok(u8_value)
    }

    pub(crate) fn get_checksum(&self) -> Result<u8> {
        // the next 8 bits
        let u8_value = Self::get_u8_from_u64(self.flit, 0);
        Ok(u8_value)
    }

    pub(crate) fn check_checksum_for_head(
        &self,
        header: Header,
        source_id: Id,
        destination_id: Id,
    ) -> Result<()> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }

        let mut sum: u8 = 0;
        sum.wrapping_add(source_id as u8);
        sum.wrapping_add(destination_id as u8);
        sum.wrapping_add(header as u8);

        anyhow::ensure!(sum == self.get_checksum()?, "checksum is not correct");
        Ok(())
    }

    // for body and tail
    pub(crate) fn get_message(&self) -> Result<[u8; 6]> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        // the next 48 bits
        let mut data: [u8; 6] = [0; 6];
        for i in 0usize..6 {
            let u8_value = Self::get_u8_from_u64(self.flit, 8 * (6 - i as u8));
            data[i] = u8_value;
        }
        let u64_value = (self.flit as u64 >> 8) & 0xFFFFFFFFFFFF;
        Ok(data)
    }

    pub(crate) fn check_checksum_for_body_and_tail(&self) -> Result<()> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        let messages = self.get_message()?;
        let mut sum: u8 = 0;
        for message in messages.iter() {
            sum.wrapping_add(*message);
        }

        // let sum = Self::get_u16_from_u64(messages, 0)
        //     + Self::get_u16_from_u64(messages, 16)
        //     + Self::get_u16_from_u64(messages, 32);
        anyhow::ensure!(sum == self.get_checksum()?, "checksum is not correct");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // todo) it is just generated by copilot

    #[test]
    fn test_flit_type() {
        let flit = 0x0000000000000000;
        let flit_type = FlitLoader::get_flit_type(flit).unwrap();
        assert_eq!(flit_type, FlitType::Head);

        let flit = 0x0000000000000001;
        let flit_type = FlitLoader::get_flit_type(flit).unwrap();
        assert_eq!(flit_type, FlitType::Body);

        let flit = 0x0000000000000002;
        let flit_type = FlitLoader::get_flit_type(flit).unwrap();
        assert_eq!(flit_type, FlitType::Tail);

        let flit = 0x0000000000000003;
        let flit_type = FlitLoader::get_flit_type(flit).unwrap();
        assert_eq!(flit_type, FlitType::Nope);
    }

    #[test]
    fn test_flit_loader_head() {
        let flit = 0x0000000000000000;
        let flit_loader = FlitLoader::new(flit).unwrap();
        assert_eq!(flit_loader.get_source_id().unwrap(), 0);
        assert_eq!(flit_loader.get_destination_id().unwrap(), 0);
        assert_eq!(flit_loader.get_header().unwrap(), Header::SendLocalnetId);
        assert_eq!(flit_loader.get_length().unwrap(), 0);
        assert_eq!(flit_loader.get_checksum().unwrap(), 0);

        let flit = 0x0000000000000100;
        let flit_loader = FlitLoader::new(flit).unwrap();
        assert_eq!(flit_loader.get_source_id().unwrap(), 0);
        assert_eq!(flit_loader.get_destination_id().unwrap(), 0);
        assert_eq!(flit_loader.get_header().unwrap(), Header::SendNodeId);
        assert_eq!(flit_loader.get_length().unwrap(), 0);
        assert_eq!(flit_loader.get_checksum().unwrap(), 1);

        let flit = 0x0000000000000200;
        let flit_loader = FlitLoader::new(flit).unwrap();
        assert_eq!(flit_loader.get_source_id().unwrap(), 0);
        assert_eq!(flit_loader.get_destination_id().unwrap(), 0);
        assert_eq!(flit_loader.get_header().unwrap(), Header::ConfirmCoordinate);
        assert_eq!(flit_loader.get_length().unwrap(), 0);
        assert_eq!(flit_loader.get_checksum().unwrap(), 2);
    }
    #[test]
    fn test_random_flit() {
        let flit = 0x0000000000000000;
    }
}
