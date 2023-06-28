use anyhow::Result;
use std::time::Duration;
use std_display::efuse::{
    Efuse, LOCALNET_DOWNLEFT, LOCALNET_DOWNRIGHT, LOCALNET_UPLEFT, LOCALNET_UPRIGHT, ROOT,
};

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let efuse = Efuse::new();

    if efuse.is_root() {
        println!("data3_7: {:x}", efuse.is_root());
        eprintln!("Data had already written: Already ROOT");
    } else {
        println!("data3_7: {:x}", efuse.is_root());
        println!("Change ROOT: {:x}", ROOT);
        efuse.write_root();
        println!("data3_7: {:x}", ROOT);
    }

    println!("Done");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
