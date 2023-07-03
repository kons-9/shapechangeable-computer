use anyhow::anyhow;
use anyhow::Result;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

/// Flit consists of 64 bits.
/// HeadFlit : [ FlitType(8) | Header(8) | SourceId(16) | DestinationId(16) | Checksum(16) ]
/// Body and TailFlit : [ FlitType(8) | Message(48) | Checksum(8)]
/// NopeFlit : [ FlitType(8) | z(undefined)(56) ]
pub type Flit = u64;
pub type Id = u16;
pub type Coordinate = (u16, u16);

const U16_MAX: u16 = 0xFFFF;
const U8_MAX: u8 = 0xFF;

#[derive(TryFromPrimitive, PartialEq, Debug)]
#[repr(u8)]
pub enum FlitType {
    Head,
    Body,
    Tail,
    Nope,
}

#[derive(TryFromPrimitive, PartialEq, Debug)]
#[repr(u8)]
pub enum Header {
    // for making localnet
    /// LocalnetId + NodeId
    SendLocalnetId,
    SendNodeId,
    GetConnectedNeighborCoordinate,
    ShareNeighborCoordinate,
    ConfirmCoordinate,

    // for making tree
    SendParentId,
    ReceiveParentId,
    SendChildId,
    ReceiveChildId,

    // for reply
    Ack,

    // for error
    HeaderError,
    FilterError,
    FromError,
    ToError,
    CheckSumError,
    EtcError,
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
        TryFrom::try_from((val >> start) & U16_MAX as u64).unwrap()
    }
    fn get_u8_from_u64(val: u64, start: u8) -> u8 {
        TryFrom::try_from((val >> start) & U8_MAX as u64).unwrap()
    }

    // for all
    fn get_flit_type(flit: Flit) -> Result<FlitType> {
        // the toppest 8 bits
        let u8_value = (flit as u64 >> 56) as u8;
        Ok(FlitType::try_from(u8_value)?)
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

    pub(crate) fn get_checksum_for_head(&self) -> Result<u16> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 16 bits
        let u16_value = Self::get_u16_from_u64(self.flit, 0);
        Ok(u16_value)
    }

    pub(crate) fn check_checksum_for_head(&self) -> Result<()> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }

        let mut sum: u16 = 0;
        sum.wrapping_add(self.get_source_id()?);
        sum.wrapping_add(self.get_destination_id()?);
        sum.wrapping_add(self.get_header()? as u16);

        anyhow::ensure!(
            sum == self.get_checksum_for_head()?,
            "checksum is not correct"
        );
        Ok(())
    }

    // for body and tail
    pub(crate) fn get_message(&self) -> Result<Vec<u8>> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        // the next 48 bits
        let mut data = Vec::new();
        for i in 0..6 {
            let u8_value = Self::get_u8_from_u64(self.flit, 8 * (6 - i));
            data.push(u8_value);
        }
        let u64_value = (self.flit as u64 >> 8) & 0xFFFFFFFFFFFF;
        Ok(data)
    }

    pub(crate) fn get_checksum_for_body_and_tail(&self) -> Result<u8> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        // the next 8 bits
        let u8_value = Self::get_u8_from_u64(self.flit, 0);
        Ok(u8_value)
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
        anyhow::ensure!(
            sum == self.get_checksum_for_body_and_tail()?,
            "checksum is not correct"
        );
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
        assert_eq!(flit_loader.get_checksum_for_head().unwrap(), 0);

        let flit = 0x0000000000000100;
        let flit_loader = FlitLoader::new(flit).unwrap();
        assert_eq!(flit_loader.get_source_id().unwrap(), 0);
        assert_eq!(flit_loader.get_destination_id().unwrap(), 0);
        assert_eq!(flit_loader.get_header().unwrap(), Header::SendNodeId);
        assert_eq!(flit_loader.get_checksum_for_head().unwrap(), 1);

        let flit = 0x0000000000000200;
        let flit_loader = FlitLoader::new(flit).unwrap();
        assert_eq!(flit_loader.get_source_id().unwrap(), 0);
        assert_eq!(flit_loader.get_destination_id().unwrap(), 0);
        assert_eq!(flit_loader.get_header().unwrap(), Header::ConfirmCoordinate);
        assert_eq!(flit_loader.get_checksum_for_head().unwrap(), 2);
    }
    #[test]
    fn test_random_flit() {
        let flit = 0x0000000000000000;
    }
}
