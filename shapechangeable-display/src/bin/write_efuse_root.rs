use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;
use std_display::efuse::Efuse;
use network_node::utils::util;
use network_node::utils::util_const::ROOT;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut efuse = Efuse::new();
    let mac_address = efuse.get_efuse_value();
    let is_root = util::is_root(mac_address);

    if is_root {
        println!("data3_7: {:x}", is_root as u32);
        eprintln!("Data had already written: Already ROOT");
    } else {
        println!("data3_7: {:x}", is_root as u32);
        println!("Change ROOT: {:x}", ROOT);
        efuse.write_root();
        efuse.update();
        let mac_address = efuse.get_efuse_value();
        let is_root = util::is_root(mac_address);
        println!("data3_7: {:x}", is_root as u32);
    }

    println!("Done");

    loop {
        sleep(Duration::from_secs(1));
    }
}
