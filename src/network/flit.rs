use anyhow::Result;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

type Flit = u64;

const U16_MAX: u16 = 0xFFFF;
const U8_MAX: u8 = 0xFF;

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum FlitType {
    Head,
    Body,
    Tail,
    NOPE,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum Header {
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

struct FlitLoader {
    flit: Flit,
    flit_type: FlitType,
}

impl FlitLoader {
    pub fn new(flit: Flit) -> Result<Self> {
        Ok(FlitLoader {
            flit,
            flit_type: FlitLoader::get_flit_type(flit)?,
        })
    }

    // utils
    fn get_u16_from_u64(val: u64, start: u8) -> u16 {
        let end = start + 16;
        TryFrom::try_from((val >> start) & U16_MAX).unwrap()
    }
    fn get_u8_from_u64(val: u64, start: u8) -> u16 {
        let end = start + 16;
        TryFrom::try_from((val >> start) & U8_MAX).unwrap()
    }

    // for all
    fn get_flit_type(flit: Flit) -> Result<FlitType> {
        // the toppest 8 bits
        let u8_value = (flit as u64 >> 56) as u8;
        FlitType::try_from(u8_value)
    }

    // for head
    pub fn get_source_id(&self) -> Result<u16> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 16 bits
        let u16_value = Self::get_u16_from_u64(self.flit, 40);
        Ok(u16_value)
    }
    pub fn get_destination_id(&self) -> Result<u16> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 16 bits
        let u16_value = Self::get_u16_from_u64(self.flit, 24);
        Ok(u16_value)
    }
    pub fn get_header(&self) -> Result<Header> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 8 bits
        let u8_value = Self::get_u8_from_u64(self.flit, 16);
        Header::try_from(u8_value)
    }

    pub fn get_checksum_for_head(&self) -> Result<u16> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        // the next 16 bits
        let u16_value = Self::get_u16_from_u64(self.flit, 0);
        Ok(u16_value)
    }

    pub fn check_checksum_for_head(&self) -> Result<()> {
        if self.flit_type != FlitType::Head {
            return Err(anyhow!("This flit is not Head"));
        }
        let sum = self.get_source_id()? + self.get_destination_id()? + self.get_header()? as u16;
        anyhow::ensure!(
            sum == self.get_checksum_for_head(),
            "checksum is not correct"
        );
        Ok(())
    }

    // for body and tail
    pub fn get_message(&self) -> Result<u64> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        // the next 48 bits
        let u64_value = (self.flit as u64 >> 8) & 0xFFFFFFFFFFFF;
        Ok(u64_value)
    }

    pub fn get_checksum_for_body_and_tail(&self) -> Result<u8> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        // the next 8 bits
        let u8_value = Self::get_u8_from_u64(self.flit, 0);
        Ok(u8_value)
    }

    pub fn check_checksum_for_body_and_tail(&self) -> Result<()> {
        if self.flit_type != FlitType::Body && self.flit_type != FlitType::Tail {
            return Err(anyhow!("This flit is not Body or Tail"));
        }
        let messages = self.get_message()?;

        let sum = Self::get_u16_from_u64(messages, 0)
            + Self::get_u16_from_u64(messages, 16)
            + Self::get_u16_from_u64(messages, 32);
        anyhow::ensure!(
            sum == self.get_checksum_for_body_and_tail(),
            "checksum is not correct"
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {}
