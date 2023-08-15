use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;
use std_display::efuse::Efuse;
use std_display::id_utils::util_const::TEMPLATE;
use std_display::id_utils::util;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut efuse = Efuse::new();
    let location = TEMPLATE;
    let mac_address = efuse.get_efuse_value();
    let localnet = util::get_raw_localnet_location(mac_address);

    if localnet != 0 {
        println!("data3_7 localnet_location: {:x}", localnet);
        eprintln!("Data had already written");
    } else {
        println!("data3_7 localnet_location: {:x}", localnet);
        println!("Change TEMPLATE: {:x}", location);
        efuse.write_localnet(location as u32);
        efuse.update();
        let mac_address = efuse.get_efuse_value();
        let localnet = util::get_raw_localnet_location(mac_address);
        println!("data3_7: {:x}", localnet);
    }

    println!("Done");

    loop {
        sleep(Duration::from_secs(1));
    }
}
