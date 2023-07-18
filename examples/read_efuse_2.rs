use anyhow::Result;
use esp32c3::Peripherals;

/// this example use esp32c3 crate, which is not used in this main lib.
fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = unsafe { Peripherals::steal() };
    let data3_7 = peripherals.EFUSE.rd_usr_data7.read().bits();

    println!("data3_7: {:x}", data3_7);

    loop {
        continue;
    }
}
