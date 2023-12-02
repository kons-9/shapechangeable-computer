use anyhow::Result;
use network_node::utils::util;
use std::thread::sleep;
use std::time::Duration;
use std_display::efuse::Efuse;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut efuse = Efuse::new();
    let mac_address: u32 = 0bTEMPLATE;
    let current_mac_address = efuse.get_efuse_value();

    println!("data3_7 current_mac_address: {:x}", current_mac_address);
    if current_mac_address != 0 {
        eprintln!("Data had already written");
    } else {
        println!("Changing mac_address into {:x}", mac_address);
        efuse.write_3_7(mac_address);
        efuse.update();
        let current_mac_address = efuse.get_efuse_value();
        println!("data3_7 current_mac_address: {:x}", current_mac_address);
        sleep(Duration::from_secs(1));
        assert_eq!(mac_address, current_mac_address as u32);
    }

    println!("Done");

    loop {
        sleep(Duration::from_secs(1));
    }
}
