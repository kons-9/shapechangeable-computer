use anyhow::Result;

pub struct Ota {
    // The OTA handle
    ota_handle: esp_idf_sys::esp_ota_handle_t,
    // The partition to write to
    partition: esp_idf_sys::esp_partition_t,
    // The offset to write to
    offset: u32,
    // The size of the partition
    size: u32,
    // The size of the data to write
    data_size: u32,
    // The data to write
    data: [u8; 1024],
}

impl Ota {
    pub fn new() -> Self {
        todo!()
    }
    pub fn connect_to_wifi(&self, wifi_ssid: &str, wifi_password: &str) -> Result<()> {
        todo!()
    }
    pub fn check_firmware_is_latest(&self, url: &str, filename: &str) -> Result<bool> {
        todo!()
    }
    pub fn download_firmware(&self, url: &str, filename: &str) -> Result<()> {
        todo!()
    }
    pub fn update_firmware(&self) -> Result<()> {
        todo!()
    }
    pub fn reboot_device(&self) -> Result<()> {
        todo!()
    }
    pub fn run(&self) -> Result<()> {
        todo!()
    }
}
