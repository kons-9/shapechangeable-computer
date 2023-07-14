use crate::id_utils::Const::*;
use crate::id_utils::TypeAlias::*;

/// efuse contains localnet_id and root information
pub struct Efuse {
    /// Block 3 (0-7)
    /// use only 3 bits of block3[7]
    block3: [u32; 8],
    // todo: we should think location.
}

impl Efuse {
    pub fn new() -> Efuse {
        // let mut block3 = Vec::new();
        let mut block3 = [0; 8];
        for i in 0..8 {
            block3[i as usize] = Self::read_reg(3, i);
        }

        Efuse { block3 }
    }
    pub fn write_root(&self) {
        Self::write(3, 7, ROOT as u32);
    }

    pub fn write_localnet(&self, localnet: u32) {
        Self::write(3, 7, localnet);
    }
    pub fn read_reg(block: u32, reg: u32) -> u32 {
        unsafe { esp_idf_sys::esp_efuse_read_reg(block, reg) }
    }

    fn write(block: u32, reg: u32, data: u32) {
        unsafe {
            esp_idf_sys::esp_efuse_write_reg(block, reg, data);
        }
    }

    // raw value of efuse
    pub fn get_mac_address(&self) -> Id {
        (self.block3[7] & ID_MASK) as Id
    }

    // pub fn from(efuse: &Efuse) -> LocalNetworkLocation {
    //     match efuse.get_raw_localnet_location() {
    //         LOCALNET_UPLEFT => LocalNetworkLocation::UpLeft,
    //         LOCALNET_UPRIGHT => LocalNetworkLocation::UpRight,
    //         LOCALNET_DOWNLEFT => LocalNetworkLocation::DownLeft,
    //         LOCALNET_DOWNRIGHT => LocalNetworkLocation::DownRight,
    //         _ => panic!(
    //             "Invalid localnet: localnet is less than 5, but {},
    //             raw_localnet_location: {},
    //             mac_address: {}",
    //             efuse.get_localnet_location(),
    //             efuse.get_raw_localnet_location(),
    //             efuse.get_mac_address()
    //         ),
    //     }
    //     // efuse.efuse_to_localnet()
    // }
    //
    // pub fn efuse_to_localnet(&self) -> LocalNetworkLocation {}
}
