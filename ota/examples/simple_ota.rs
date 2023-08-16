use anyhow::Result;
// use ota::ota::Ota;

// download the firmware from the server by http

#[derive(Debug)]
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
    println!("{:?}", config);
    println!("{:?}", std::env::var("CARGO_PKG_NAME"));
    println!("url: {}", config.url);
    println!("filename: {}", config.filename);
    println!("wifi_ssid: {}", config.wifi_ssid);
    println!("wifi_password: {}", config.wifi_password);
}
