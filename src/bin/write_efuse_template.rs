use anyhow::Result;
use std::time::Duration;
use std_display::efuse::{Efuse, TEMPLATE};

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let efuse = Efuse::new();
    let location = TEMPLATE;

    if efuse.get_localnet() != 0 {
        println!("data3_7: {:x}", efuse.get_localnet());
        eprintln!("Data had already written");
    } else {
        println!("data3_7: {:x}", efuse.get_localnet());
        println!("Change TEMPLATE: {:x}", location);
        efuse.write_root();
        println!("data3_7: {:x}", location);
    }

    println!("Done");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
