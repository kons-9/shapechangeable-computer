use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
pub mod ota;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    url: &'static str,
    #[default("")]
    filename: &'static str,
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
}

fn main() {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = CONFIG;
    println!("url: {}", config.url);
    println!("filename: {}", config.filename);
    println!("wifi_ssid: {}", config.wifi_ssid);
    println!("wifi_password: {}", config.wifi_password);
}
