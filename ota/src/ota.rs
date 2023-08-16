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
