use anyhow::Result;

use esp_idf_sys::*;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    const NEW_APP: &[u8] = include_bytes!("../../asset/bootloader.bin");

    unsafe {
        let esp_partition = esp_ota_get_boot_partition();
        let running = esp_ota_get_running_partition();
        let update_partition = esp_ota_get_next_update_partition(core::ptr::null());
        let mut update_handle = 0;

        esp_ota_begin(
            update_partition,
            OTA_SIZE_UNKNOWN as usize,
            &mut update_handle,
        );

        for app_chunk in NEW_APP.chunks(1024) {
            let err = esp_ota_write(
                update_handle,
                app_chunk.as_ptr() as *const _,
                app_chunk.len(),
            );
            if err != ESP_OK as i32 {
                panic!("esp_ota_write failed: {}", err);
            }
        }
        esp_ota_end(update_handle);
        esp_ota_set_boot_partition(update_partition);
        esp_restart();
    }
    Ok(())
}
