use anyhow::Result;
use log::info;
use ota::ota::Ota;

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

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = CONFIG;

    info!("try to connect to wifi...");
    info!("wifi_ssid: {}", config.wifi_ssid);
    info!("wifi_password: {}", config.wifi_password);
    // connect to wifi
    // check ssid and password is valid, then connect to wifi
    let ota = Ota::new();
    ota.connect_to_wifi(config.wifi_ssid, config.wifi_password)?;

    info!("connect to wifi success!");

    info!("try to download firmware...");
    info!("url: {}", config.url);
    info!("filename: {}", config.filename);
    // download firmware
    // check url is valid, then download firmware to flash(ota partition)
    if ota.check_firmware_is_latest(config.url, config.filename)? {
        info!("Firmware is latest!, no need to update!");
        return Ok(());
    }
    ota.download_firmware(config.url, config.filename)?;

    info!("download firmware success!");
    info!("try to update firmware...");
    // update firmware
    ota.update_firmware()?;

    info!("update firmware success!");
    info!("reboot device...");
    ota.reboot_device()?;

    loop {
        // wait for reboot
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
