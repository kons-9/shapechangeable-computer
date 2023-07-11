use crate::network::localnet::LocalNetworkLocation;

// ------------------------------
// const
// ------------------------------

// ID type is u16, so I use lower order 16 bits of u32
const ROOT_MASK: u32 = 0x0000_0001;
const ROOT_SHIFT: u32 = 0;
const LOCALNET_LOCATION_MASK: u32 = 0b0000_0000_0000_0110;
const LOCALNET_LOCATION_SHIFT: u32 = 1;
const LOCALNET_ID_MASK: u32 = 0b1111_1111_1111_1000;
const LOCALNET_ID_SHIFT: u32 = 3;
const MAC_ADDRESS_MASK: u32 = 0b1111_1111_1111_1111;
const MAC_ADDRESS_SHIFT: u32 = 0;

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

    // raw value of efuse
    pub fn get_raw_localnet_id(&self) -> u32 {
        self.block3[7] & LOCALNET_ID_MASK
    }
    pub fn get_raw_mac_address(&self) -> u32 {
        self.block3[7] & MAC_ADDRESS_MASK
    }
    pub fn get_raw_localnet_location(&self) -> u32 {
        self.block3[7] & LOCALNET_LOCATION_MASK
    }
    pub fn get_raw_root(&self) -> u32 {
        self.block3[7] & ROOT_MASK
    }

    // convenient value of efuse
    pub fn get_localnet_id(&self) -> u32 {
        (self.block3[7] & LOCALNET_ID_MASK) >> LOCALNET_ID_SHIFT
    }
    pub fn get_mac_address(&self) -> u32 {
        (self.block3[7] & MAC_ADDRESS_MASK) >> MAC_ADDRESS_SHIFT
    }
    pub fn get_localnet_location(&self) -> u32 {
        (self.block3[7] & LOCALNET_LOCATION_MASK) >> LOCALNET_LOCATION_SHIFT
    }
    pub fn is_root(&self) -> bool {
        (self.block3[7] & ROOT_MASK) >> ROOT_SHIFT == ROOT
    }
    pub fn from(efuse: &Efuse) -> LocalNetworkLocation {
        match efuse.get_raw_localnet_location() {
            LOCALNET_UPLEFT => LocalNetworkLocation::UpLeft,
            LOCALNET_UPRIGHT => LocalNetworkLocation::UpRight,
            LOCALNET_DOWNLEFT => LocalNetworkLocation::DownLeft,
            LOCALNET_DOWNRIGHT => LocalNetworkLocation::DownRight,
            _ => panic!(
                "Invalid localnet: localnet is less than 5, but {}, 
                raw_localnet_location: {},
                mac_address: {}",
                efuse.get_localnet_location(),
                efuse.get_raw_localnet_location(),
                efuse.get_mac_address()
            ),
        }
        // efuse.efuse_to_localnet()
    }

    // pub fn efuse_to_localnet(&self) -> LocalNetworkLocation {}
}
