use core::ffi::c_void;

use esp_idf_sys::esp_err_t;

use network_node::system::SystemInfo;
use network_node::utils::{type_alias::*, util_const::*};

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
    pub fn update(&mut self) {
        for i in 0..8 {
            self.block3[i as usize] = Self::read_reg(3, i);
        }
    }
    pub fn write_3_7(&self, data: u32) {
        Self::write(3, 7, data);
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

    fn write(block: u32, reg: u32, data: u32) -> esp_err_t {
        unsafe { esp_idf_sys::esp_efuse_write_reg(block, reg, data) }
    }

    pub fn write_block(&self, blk: u32, src: &[u32], offset: usize, len: usize) -> esp_err_t {
        unsafe {
            esp_idf_sys::esp_efuse_write_block(blk, src.as_ptr() as *const c_void, offset, len)
        }
    }

    // raw value of efuse
    pub fn get_efuse_value(&self) -> Id {
        (self.block3[7] & ID_MASK) as Id
    }
}
impl SystemInfo for Efuse {
    fn get_system_info(&self) -> Id {
        self.get_efuse_value()
    }
}
