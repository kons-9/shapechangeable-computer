use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_sys::esp_efuse_desc_t;
use std_display::efuse::Efuse;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let data2_0 = Efuse::read_reg(2, 0);

    let data2_1 = Efuse::read_reg(2, 1);

    let data3_0 = Efuse::read_reg(3, 0);

    let data3_7 = Efuse::read_reg(3, 7);

    println!("data2_0: {:x}", data2_0);
    println!("data2_1: {:x}", data2_1);
    println!("data3_0: {:x}", data3_0);
    println!("data3_7: {:x}", data3_7);

    // write efuse 3-7
    // let data3_7 = 0x00000001; // root

    // unsafe {
    //     esp_idf_sys::esp_efuse_write_reg(3, 7, data3_7);
    // }

    let data3_7 = Efuse::read_reg(3, 7);
    println!("data3_7: {:x}", data3_7);

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
