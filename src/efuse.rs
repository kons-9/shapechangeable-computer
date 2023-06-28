use crate::localnet::LocalNetworkLocation;

// ------------------------------
// const
// ------------------------------

const ROOT_MASK: u32 = 0x00000001;
const LOCALNET_MASK: u32 = 0x00000006;

pub const ROOT: u32 = 0x00000001;
pub const LOCALNET_UPLEFT: u32 = 0x00000000;
pub const LOCALNET_UPRIGHT: u32 = 0x00000002;
pub const LOCALNET_DOWNLEFT: u32 = 0x00000004;
pub const LOCALNET_DOWNRIGHT: u32 = 0x00000006;

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
        unsafe {
            for i in 0..8 {
                block3[i as usize] = esp_idf_sys::esp_efuse_read_reg(3, i);
            }
        }

        Efuse { block3 }
    }
    pub fn write_root(&self) {
        self.write(3, 7, ROOT);
    }

    pub fn write_localnet(&self, localnet: u32) {
        self.write(3, 7, localnet);
    }

    fn write(&self, block: u32, reg: u32, data: u32) {
        unsafe {
            esp_idf_sys::esp_efuse_write_reg(block, reg, data);
        }
    }

    pub fn get_localnet(&self) -> u32 {
        self.block3[7] & LOCALNET_MASK
    }
    pub fn is_root(&self) -> bool {
        self.block3[7] & ROOT_MASK == ROOT
    }

    pub fn efuse_to_localnet(&self) -> LocalNetworkLocation {
        match self.get_localnet() {
            LOCALNET_UPLEFT => LocalNetworkLocation::UpLeft,
            LOCALNET_UPRIGHT => LocalNetworkLocation::UpRight,
            LOCALNET_DOWNLEFT => LocalNetworkLocation::DownLeft,
            LOCALNET_DOWNRIGHT => LocalNetworkLocation::DownRight,
            _ => panic!(
                "Invalid localnet: localnet is less than 5, but {}",
                self.get_localnet()
            ),
        }
    }
}
