use anyhow::Result;

use esp_idf_sys::*;
use log::info;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    const NEW_APP: &[u8] = include_bytes!("../../asset/bootloader.bin");

    unsafe {
        println!("Starting OTA");
        let esp_partition = esp_ota_get_boot_partition();
        let running = esp_ota_get_running_partition();
        info!("esp_partition: {:x?}", *esp_partition);
        info!("running: {:x?}", *running);

        let update_partition = esp_ota_get_next_update_partition(core::ptr::null());
        info!("update_partition: {:x?}", *update_partition);

        if update_partition.is_null() {
            panic!("esp_ota_get_next_update_partition failed");
        }

        let mut update_handle = 0;

        let err = esp_ota_begin(update_partition, 0 as usize, &mut update_handle);
        if err != ESP_OK as i32 {
            panic!("esp_ota_begin failed: {}", err);
        }

        info!("update_handle: {:x?}", update_handle);

        if update_handle == 0 {
            panic!("esp_ota_begin failed");
        }

        for app_chunk in NEW_APP.chunks(4096*8) {
            println!("len: {}", app_chunk.len());

            let err = esp_ota_write(
                update_handle,
                app_chunk.as_ptr() as *const _,
                app_chunk.len(),
            );
            if err != ESP_OK as i32 {
                panic!("esp_ota_write failed: {}", err);
            }
        }
        let err = esp_ota_end(update_handle);
        if err != ESP_OK as i32 {
            panic!("esp_ota_end failed: {}", err);
        }

        let err = esp_ota_set_boot_partition(update_partition);
        if err != ESP_OK as i32 {
            panic!("esp_ota_set_boot_partition failed: {}", err);
        }

        esp_restart();
    }
    Ok(())
}
